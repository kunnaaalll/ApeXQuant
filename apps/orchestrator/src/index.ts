import pino from 'pino';
import { createSchema, createYoga } from 'graphql-yoga';
import { createServer } from 'http';

const logger = pino();
const port = process.env.PORT || 3002;

const schema = createSchema({
  typeDefs: `
    type Query {
      status: String!
      workflows: [String!]!
    }
  `,
  resolvers: {
    Query: {
      status: () => 'Orchestrator active',
      workflows: () => ['signal_reconciliation_workflow', 'portfolio_rebalance_workflow']
    }
  }
});

const yoga = createYoga({ schema });
const server = createServer(yoga);

server.listen(port, () => {
  logger.info(`APEX V3 Orchestrator listening on port ${port}`);
});
