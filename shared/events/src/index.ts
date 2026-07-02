import { Redis } from 'ioredis';
import { events } from '@apex-v3/contracts';

export class EventClient {
  private redis: Redis;

  constructor(redisUrl?: string) {
    this.redis = new Redis(redisUrl || process.env.REDIS_URL || 'redis://localhost:6379');
  }

  async publish(topic: string, event: events.Event): Promise<void> {
    const payload = JSON.stringify(event);
    await this.redis.xadd(topic, '*', 'event', payload);
  }

  async subscribe(topic: string, group: string, consumer: string, onEvent: (event: events.Event) => void): Promise<void> {
    try {
      await this.redis.xgroup('CREATE', topic, group, '$', 'MKSTREAM');
    } catch (err: any) {
      // Ignore group already exists
    }

    const poll = async () => {
      while (true) {
        try {
          const results = await this.redis.xreadgroup('GROUP', group, consumer, 'COUNT', '1', 'BLOCK', '5000', 'STREAMS', topic, '>') as any;
          if (results && Array.isArray(results)) {
            for (const streamResult of results) {
              const messages = streamResult[1];
              if (Array.isArray(messages)) {
                for (const message of messages) {
                  const id = message[0];
                  const keyValues = message[1];
                  if (Array.isArray(keyValues) && keyValues.length >= 2) {
                    const payload = keyValues[1];
                    const parsed = JSON.parse(payload as string) as events.Event;
                    onEvent(parsed);
                    await this.redis.xack(topic, group, id);
                  }
                }
              }
            }
          }
        } catch (err) {
          console.error(`Error in event subscriber for ${topic}:`, err);
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
      }
    };
    poll();
  }

  async close(): Promise<void> {
    await this.redis.quit();
  }
}
export { events };
