import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import { pinoHttp } from 'pino-http';
import pino from 'pino';
import dotenv from 'dotenv';
import { register } from 'prom-client';
import * as grpc from '@grpc/grpc-js';

dotenv.config();

const logger = pino();
const app = express();
const port = process.env.PORT || 3001;

// Engine gRPC endpoints (from env or default local ports)
const ENGINE_URLS = {
  execution: process.env.EXECUTION_ENGINE_URL || 'localhost:50051',
  risk:      process.env.RISK_ENGINE_URL       || 'localhost:50052',
  signal:    process.env.SIGNAL_ENGINE_URL     || 'localhost:50053',
  position:  process.env.POSITION_ENGINE_URL   || 'localhost:50055',
  portfolio: process.env.PORTFOLIO_ENGINE_URL  || 'localhost:50056',
};

// Helper: probe if a gRPC channel can reach the endpoint
function probeGrpcChannel(address: string): Promise<boolean> {
  return new Promise((resolve) => {
    const channel = new grpc.Channel(
      address,
      grpc.credentials.createInsecure(),
      {}
    );
    const deadline = new Date(Date.now() + 800);
    channel.watchConnectivityState(
      channel.getConnectivityState(true),
      deadline,
      () => {
        const state = channel.getConnectivityState(false);
        resolve(state === grpc.connectivityState.READY ||
                state === grpc.connectivityState.IDLE);
        channel.close();
      }
    );
  });
}

app.use(helmet());
app.use(cors());
app.use(express.json());
app.use(pinoHttp({ logger }));

const limiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 100,
  message: 'Too many requests from this IP, please try again later'
});
app.use(limiter);

app.get('/health', (_req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    service: 'api-gateway',
    version: '3.0.0'
  });
});

app.get('/metrics', async (_req, res) => {
  try {
    res.set('Content-Type', register.contentType);
    res.end(await register.metrics());
  } catch (err) {
    res.status(500).end(err);
  }
});

app.get('/api/v3/status', (_req, res) => {
  res.json({
    system: 'APEX V3',
    uptime: process.uptime(),
    environment: process.env.NODE_ENV || 'development'
  });
});

// gRPC connectivity probe — returns live reachability for each backend engine
app.get('/api/v3/engines', async (_req, res) => {
  const results = await Promise.all(
    Object.entries(ENGINE_URLS).map(async ([name, url]) => {
      const reachable = await probeGrpcChannel(url);
      return [name, { url, status: reachable ? 'ONLINE' : 'OFFLINE' }];
    })
  );
  res.json(Object.fromEntries(results));
});

app.listen(port, () => {
  logger.info(`APEX V3 API Gateway listening on port ${port}`);
  logger.info('Engine gRPC endpoints configured:', ENGINE_URLS);
});
