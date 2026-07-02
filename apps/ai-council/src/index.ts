import pino from 'pino';

const logger = pino();

logger.info(`APEX V3 AI Council initialized`);

const interval = setInterval(() => {
  logger.info(`AI Council heartbeat: Monitoring signal validation queue`);
}, 30000);

process.on('SIGTERM', () => {
  clearInterval(interval);
  logger.info('AI Council shutting down');
});
