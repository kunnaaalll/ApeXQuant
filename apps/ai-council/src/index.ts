import pino from 'pino';
import * as grpc from '@grpc/grpc-js';
import * as proto from '@grpc/proto-loader';
import path from 'path';
import { fileURLToPath } from 'url';

const logger = pino();

const EVENT_BUS_URL = process.env.EVENT_BUS_URL || 'localhost:50050';
const PROTO_PATH = process.env.EVENTS_PROTO_PATH ||
  path.resolve(fileURLToPath(import.meta.url), '../../../../../../shared/protobuf/events.proto');

logger.info(`APEX V3 AI Council initialized — connecting to EventBus at ${EVENT_BUS_URL}`);

// Subscribe to the event bus via gRPC streaming
async function subscribeToEventBus(): Promise<void> {
  try {
    const packageDef = proto.loadSync(PROTO_PATH, {
      keepCase: true,
      longs: String,
      enums: String,
      defaults: true,
      oneofs: true,
    });
    const protoDescriptor = grpc.loadPackageDefinition(packageDef) as any;
    const EventBusService = protoDescriptor?.apex?.events?.EventBusService;

    if (!EventBusService) {
      logger.warn('EventBusService not found in proto descriptor — falling back to heartbeat mode');
      return startHeartbeat();
    }

    const client = new EventBusService(
      EVENT_BUS_URL,
      grpc.credentials.createInsecure()
    );

    const subscribeReq = {
      consumer_group: 'ai-council',
      consumer_id: `ai-council-${process.pid}`,
      topics: ['signal.generated', 'execution.order', 'execution.position'],
      batch_size: 10,
    };

    logger.info('Subscribing to event bus topics:', subscribeReq.topics);
    const stream = client.Subscribe(subscribeReq);

    stream.on('data', (batch: any) => {
      const events: any[] = batch.events || [];
      for (const event of events) {
        const topic: string = event.topic || 'unknown';
        const eventType: string = event.event_type || 'unknown';
        logger.info({ topic, eventType }, 'AI Council received event');

        // Validate signal events through the council
        if (topic === 'signal.generated') {
          handleSignalEvent(event);
        }
      }
    });

    stream.on('error', (err: Error) => {
      logger.warn({ err: err.message }, 'EventBus stream error — will retry in 10s');
      setTimeout(subscribeToEventBus, 10_000);
    });

    stream.on('end', () => {
      logger.info('EventBus stream ended — reconnecting in 5s');
      setTimeout(subscribeToEventBus, 5_000);
    });
  } catch (err) {
    logger.warn({ err }, 'Failed to connect to EventBus — running in heartbeat mode');
    startHeartbeat();
  }
}

function handleSignalEvent(event: any): void {
  // AI Council validation: log signal for review
  // Future: fan-out to LLM providers for validation vote
  logger.info({
    signal_id: event.event_id?.value,
    source: event.source_service,
  }, 'AI Council: signal received for validation');
}

function startHeartbeat(): void {
  const interval = setInterval(() => {
    logger.info('AI Council heartbeat: Monitoring signal validation queue');
  }, 30_000);

  process.on('SIGTERM', () => {
    clearInterval(interval);
    logger.info('AI Council shutting down');
    process.exit(0);
  });
}

// Start the subscription loop
subscribeToEventBus();
