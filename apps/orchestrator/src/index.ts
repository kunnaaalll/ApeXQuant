import pino from 'pino';
import { createSchema, createYoga } from 'graphql-yoga';
import { createServer } from 'http';
import * as grpc from '@grpc/grpc-js';

const logger = pino();
const port = process.env.PORT || 3002;

// Engine gRPC channels
function makeChannel(envVar: string, defaultAddr: string) {
  return new grpc.Channel(
    process.env[envVar] || defaultAddr,
    grpc.credentials.createInsecure(),
    {}
  );
}

const engineChannels = {
  execution: makeChannel('EXECUTION_ENGINE_URL', 'localhost:50051'),
  risk:      makeChannel('RISK_ENGINE_URL',       'localhost:50052'),
  signal:    makeChannel('SIGNAL_ENGINE_URL',     'localhost:50053'),
  position:  makeChannel('POSITION_ENGINE_URL',   'localhost:50055'),
  portfolio: makeChannel('PORTFOLIO_ENGINE_URL',  'localhost:50056'),
};

function getChannelState(channel: grpc.Channel): string {
  const state = channel.getConnectivityState(true);
  switch (state) {
    case grpc.connectivityState.READY:       return 'READY';
    case grpc.connectivityState.IDLE:        return 'IDLE';
    case grpc.connectivityState.CONNECTING:  return 'CONNECTING';
    case grpc.connectivityState.TRANSIENT_FAILURE: return 'DEGRADED';
    case grpc.connectivityState.SHUTDOWN:    return 'SHUTDOWN';
    default: return 'UNKNOWN';
  }
}

const schema = createSchema({
  typeDefs: `
    type EngineStatus {
      name: String!
      status: String!
      address: String!
    }

    type WorkflowStatus {
      name: String!
      description: String!
      active: Boolean!
    }

    type Query {
      status: String!
      engines: [EngineStatus!]!
      workflows: [WorkflowStatus!]!
    }
  `,
  resolvers: {
    Query: {
      status: () => 'Orchestrator active',

      engines: () => {
        return Object.entries(engineChannels).map(([name, channel]) => ({
          name,
          status: getChannelState(channel),
          address: process.env[`${name.toUpperCase()}_ENGINE_URL`] || `localhost:5005${Object.keys(engineChannels).indexOf(name) + 1}`,
        }));
      },

      workflows: () => [
        {
          name: 'signal_reconciliation_workflow',
          description: 'Validates signals from Signal Engine against Risk Engine before execution',
          active: true,
        },
        {
          name: 'portfolio_rebalance_workflow',
          description: 'Triggers portfolio rebalancing based on allocation drift',
          active: true,
        },
        {
          name: 'position_recovery_workflow',
          description: 'Reconciles local position state against broker on startup',
          active: true,
        },
      ],
    },
  },
});

const yoga = createYoga({ schema, logging: { debug: false, info: false, warn: true, error: true } });
const server = createServer(yoga);

server.listen(port, () => {
  logger.info(`APEX V3 Orchestrator listening on port ${port}`);
  logger.info('gRPC engine channels initialized');
});
