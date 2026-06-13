import Redis from 'ioredis';
import * as msgpack from 'msgpack-lite';

// Redis connection configuration
export interface RedisConfig {
  host: string;
  port: number;
  password?: string;
  db?: number;
  maxRetriesPerRequest?: number;
  enableReadyCheck?: boolean;
}

// Connection factory for different Redis use cases
export class RedisConnections {
  private static instances: Map<string, Redis> = new Map();

  static getConnection(name: string, config: RedisConfig): Redis {
    if (this.instances.has(name)) {
      return this.instances.get(name)!;
    }

    const redis = new Redis({
      host: config.host,
      port: config.port,
      password: config.password,
      db: config.db ?? 0,
      maxRetriesPerRequest: config.maxRetriesPerRequest ?? 3,
      enableReadyCheck: config.enableReadyCheck ?? true,
      retryStrategy: (times) => {
        const delay = Math.min(times * 50, 2000);
        return delay;
      },
    });

    this.instances.set(name, redis);
    return redis;
  }

  static async closeAll(): Promise<void> {
    for (const [name, redis] of this.instances) {
      await redis.quit();
    }
    this.instances.clear();
  }
}

// ==================== STREAM CONSUMERS ====================

export interface StreamMessage {
  id: string;
  data: Record<string, string>;
}

export class StreamConsumer {
  private redis: Redis;
  private consumerGroup: string;
  private consumerName: string;
  private isRunning = false;

  constructor(
    redis: Redis,
    private streamKey: string,
    consumerGroup: string,
    consumerName: string
  ) {
    this.redis = redis;
    this.consumerGroup = consumerGroup;
    this.consumerName = consumerName;
  }

  async initialize(): Promise<void> {
    try {
      await this.redis.xgroup(
        'CREATE',
        this.streamKey,
        this.consumerGroup,
        '$',
        'MKSTREAM'
      );
    } catch (err: any) {
      if (!err.message.includes('BUSYGROUP')) {
        throw err;
      }
    }
  }

  async read(
    count: number = 10,
    blockMs: number = 5000
  ): Promise<StreamMessage[]> {
    const results = await this.redis.xreadgroup(
      'GROUP',
      this.consumerGroup,
      this.consumerName,
      'COUNT',
      count,
      'BLOCK',
      blockMs,
      'STREAMS',
      this.streamKey,
      '>'
    );

    if (!results) return [];

    const messages: StreamMessage[] = [];
    for (const [, entries] of results as [string, [string, string[]][]][]) {
      for (const [id, fields] of entries) {
        const data: Record<string, string> = {};
        for (let i = 0; i < fields.length; i += 2) {
          data[fields[i]] = fields[i + 1];
        }
        messages.push({ id, data });
      }
    }

    return messages;
  }

  async ack(messageId: string): Promise<void> {
    await this.redis.xack(this.streamKey, this.consumerGroup, messageId);
  }

  async claimPending(
    minIdleTime: number,
    count: number = 10
  ): Promise<StreamMessage[]> {
    const results = await this.redis.xpending(
      this.streamKey,
      this.consumerGroup,
      '-',
      '+',
      count
    );

    if (!results || results.length === 0) return [];

    const ids = results.map((r: string[]) => r[0]);
    if (ids.length === 0) return [];

    const claimed = await this.redis.xclaim(
      this.streamKey,
      this.consumerGroup,
      this.consumerName,
      minIdleTime,
      ...ids
    );

    const messages: StreamMessage[] = [];
    for (const [id, fields] of claimed as [string, string[]][]) {
      if (!fields) continue;
      const data: Record<string, string> = {};
      for (let i = 0; i < fields.length; i += 2) {
        data[fields[i]] = fields[i + 1];
      }
      messages.push({ id, data });
    }

    return messages;
  }

  async stop(): Promise<void> {
    this.isRunning = false;
  }
}

export class StreamProducer {
  constructor(private redis: Redis) {}

  async produce(
    streamKey: string,
    data: Record<string, any>,
    maxlen?: number
  ): Promise<string> {
    const serialized: Record<string, string> = {};
    for (const [key, value] of Object.entries(data)) {
      if (typeof value === 'object') {
        serialized[key] = JSON.stringify(value);
      } else {
        serialized[key] = String(value);
      }
    }

    const args: (string | number)[] = ['XADD', streamKey];
    if (maxlen) {
      args.push('MAXLEN', '~', maxlen);
    }
    args.push('*');

    for (const [key, value] of Object.entries(serialized)) {
      args.push(key, value);
    }

    return this.redis.sendCommand(new (require('ioredis').Command)(...args));
  }

  async produceBatch(
    streamKey: string,
    messages: Record<string, any>[],
    maxlen?: number
  ): Promise<string[]> {
    const pipeline = this.redis.pipeline();

    for (const data of messages) {
      const serialized: Record<string, string> = {};
      for (const [key, value] of Object.entries(data)) {
        if (typeof value === 'object') {
          serialized[key] = JSON.stringify(value);
        } else {
          serialized[key] = String(value);
        }
      }

      const args: (string | number)[] = ['XADD', streamKey];
      if (maxlen) {
        args.push('MAXLEN', '~', maxlen);
      }
      args.push('*');

      for (const [key, value] of Object.entries(serialized)) {
        args.push(key, value);
      }

      pipeline.sendCommand(new (require('ioredis').Command)(...args));
    }

    const results = await pipeline.exec();
    return results?.map((r) => r[1] as string) ?? [];
  }
}

// ==================== TEMPORARY STATE ====================

export interface StateConfig {
  prefix: string;
  ttlSeconds: number;
}

export class TemporaryState {
  constructor(
    private redis: Redis,
    private config: StateConfig
  ) {}

  private key(id: string): string {
    return `${this.config.prefix}:${id}`;
  }

  async set<T>(id: string, state: T): Promise<void> {
    const key = this.key(id);
    const serialized = msgpack.encode(state);
    await this.redis.setex(key, this.config.ttlSeconds, serialized);
  }

  async get<T>(id: string): Promise<T | null> {
    const key = this.key(id);
    const data = await this.redis.getBuffer(key);
    if (!data) return null;
    return msgpack.decode(data) as T;
  }

  async delete(id: string): Promise<void> {
    await this.redis.del(this.key(id));
  }

  async exists(id: string): Promise<boolean> {
    const result = await this.redis.exists(this.key(id));
    return result === 1;
  }

  async extend(id: string): Promise<void> {
    await this.redis.expire(this.key(id), this.config.ttlSeconds);
  }
}

// ==================== CACHING ====================

export class Cache {
  constructor(private redis: Redis) {}

  async get<T>(key: string): Promise<T | null> {
    const data = await this.redis.get(key);
    if (!data) return null;
    try {
      return JSON.parse(data) as T;
    } catch {
      return data as unknown as T;
    }
  }

  async set<T>(
    key: string,
    value: T,
    ttlSeconds?: number
  ): Promise<void> {
    const serialized = typeof value === 'string' ? value : JSON.stringify(value);
    if (ttlSeconds) {
      await this.redis.setex(key, ttlSeconds, serialized);
    } else {
      await this.redis.set(key, serialized);
    }
  }

  async delete(key: string): Promise<void> {
    await this.redis.del(key);
  }

  async getOrSet<T>(
    key: string,
    factory: () => Promise<T>,
    ttlSeconds?: number
  ): Promise<T> {
    const cached = await this.get<T>(key);
    if (cached !== null) {
      return cached;
    }

    const value = await factory();
    await this.set(key, value, ttlSeconds);
    return value;
  }

  async invalidatePattern(pattern: string): Promise<void> {
    const keys = await this.redis.keys(pattern);
    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
  }
}

// ==================== RATE LIMITING ====================

export interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
}

export class RateLimiter {
  constructor(private redis: Redis) {}

  async checkLimit(
    key: string,
    config: RateLimitConfig
  ): Promise<{ allowed: boolean; remaining: number; resetAt: number }> {
    const now = Date.now();
    const windowStart = now - config.windowMs;

    const pipeline = this.redis.pipeline();
    pipeline.zremrangebyscore(key, 0, windowStart);
    pipeline.zcard(key);
    pipeline.zadd(key, now, `${now}-${Math.random()}`);
    pipeline.pexpire(key, config.windowMs);

    const results = await pipeline.exec();
    const currentCount = (results?.[1]?.[1] as number) ?? 0;

    const allowed = currentCount < config.maxRequests;
    const remaining = Math.max(0, config.maxRequests - currentCount - 1);
    const resetAt = now + config.windowMs;

    if (!allowed) {
      await this.redis.zrem(key, `${now}-${Math.random()}`);
    }

    return { allowed, remaining, resetAt };
  }
}

// ==================== PUB/SUB ====================

export type MessageHandler = (channel: string, message: string) => void;

export class PubSub {
  private subscriber: Redis;
  private handlers: Map<string, Set<MessageHandler>> = new Map();

  constructor(private redis: Redis) {
    this.subscriber = redis.duplicate();
  }

  async subscribe(channel: string, handler: MessageHandler): Promise<void> {
    if (!this.handlers.has(channel)) {
      this.handlers.set(channel, new Set());
      await this.subscriber.subscribe(channel);
    }

    this.handlers.get(channel)!.add(handler);

    this.subscriber.on('message', (receivedChannel, message) => {
      const channelHandlers = this.handlers.get(receivedChannel);
      if (channelHandlers) {
        channelHandlers.forEach((h) => h(receivedChannel, message));
      }
    });
  }

  async unsubscribe(channel: string, handler: MessageHandler): Promise<void> {
    const channelHandlers = this.handlers.get(channel);
    if (channelHandlers) {
      channelHandlers.delete(handler);
      if (channelHandlers.size === 0) {
        await this.subscriber.unsubscribe(channel);
        this.handlers.delete(channel);
      }
    }
  }

  async publish(channel: string, message: string | object): Promise<void> {
    const payload = typeof message === 'string' ? message : JSON.stringify(message);
    await this.redis.publish(channel, payload);
  }

  async close(): Promise<void> {
    await this.subscriber.quit();
  }
}

// ==================== LEADERBOARD (sorted sets) ====================

export class Leaderboard {
  constructor(private redis: Redis) {}

  async addScore(key: string, member: string, score: number): Promise<void> {
    await this.redis.zadd(key, score, member);
  }

  async getRank(key: string, member: string): Promise<number | null> {
    const rank = await this.redis.zrevrank(key, member);
    return rank !== null ? rank + 1 : null;
  }

  async getTop(key: string, count: number): Promise<{ member: string; score: number }[]> {
    const results = await this.redis.zrevrange(key, 0, count - 1, 'WITHSCORES');
    const pairs: { member: string; score: number }[] = [];
    for (let i = 0; i < results.length; i += 2) {
      pairs.push({ member: results[i], score: parseFloat(results[i + 1]) });
    }
    return pairs;
  }

  async getScore(key: string, member: string): Promise<number | null> {
    const score = await this.redis.zscore(key, member);
    return score !== null ? parseFloat(score) : null;
  }

  async incrementScore(key: string, member: string, increment: number): Promise<number> {
    return this.redis.zincrby(key, increment, member);
  }
}

// ==================== EXPORTS ====================

export { Redis, msgpack };

export * from './streams';
export * from './state';
