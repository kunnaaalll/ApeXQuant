import { NextResponse } from 'next/server';
import net from 'net';

// Engine port map: service -> default gRPC port
const ENGINE_PORTS: Record<string, number> = {
  event_bus:        50050,
  execution_engine: 50051,
  risk_engine:      50052,
  signal_engine:    50053,
  performance_engine: 50054,
  position_engine:  50055,
  portfolio_engine: 50056,
  analytics_engine: 50057,
  learning_engine:  50058,
  mt5_bridge:       8000,
};

function checkPort(host: string, port: number, timeout = 500): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = new net.Socket();
    socket.setTimeout(timeout);

    socket.on('connect', () => {
      socket.destroy();
      resolve(true);
    });

    socket.on('timeout', () => {
      socket.destroy();
      resolve(false);
    });

    socket.on('error', () => {
      socket.destroy();
      resolve(false);
    });

    socket.connect(port, host);
  });
}

export async function GET() {
  const host = process.env.ENGINE_HOST || '127.0.0.1';

  const checks = await Promise.all(
    Object.entries(ENGINE_PORTS).map(async ([name, port]) => {
      const online = await checkPort(host, port);
      return [name, online ? 'ONLINE' : 'OFFLINE'] as const;
    })
  );

  return NextResponse.json(Object.fromEntries(checks));
}
