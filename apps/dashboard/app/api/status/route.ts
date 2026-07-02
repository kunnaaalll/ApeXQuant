import { NextResponse } from 'next/server';
import net from 'net';
import { exec } from 'child_process';

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

function getRunningProcesses(): Promise<string> {
  return new Promise((resolve) => {
    exec('tasklist', (err, stdout) => {
      if (err) {
        resolve('');
        return;
      }
      resolve(stdout.toLowerCase());
    });
  });
}

export async function GET() {
  const host = '127.0.0.1';
  
  // Concurrently fetch the process list and ping the MT5 bridge port
  const [processList, mt5BridgeOnline] = await Promise.all([
    getRunningProcesses(),
    checkPort(host, 8000)
  ]);
  
  return NextResponse.json({
    event_bus: processList.includes('event-bus.exe') ? 'ONLINE' : 'OFFLINE',
    portfolio_engine: processList.includes('portfolio-engine.exe') ? 'ONLINE' : 'OFFLINE',
    execution_engine: processList.includes('execution-engine.exe') ? 'ONLINE' : 'OFFLINE',
    risk_engine: processList.includes('risk-engine.exe') ? 'ONLINE' : 'OFFLINE',
    signal_engine: processList.includes('signal-engine.exe') ? 'ONLINE' : 'OFFLINE',
    position_engine: processList.includes('position-engine.exe') ? 'ONLINE' : 'OFFLINE',
    analytics_engine: processList.includes('analytics-engine.exe') ? 'ONLINE' : 'OFFLINE',
    learning_engine: processList.includes('learning-engine.exe') ? 'ONLINE' : 'OFFLINE',
    mt5_bridge: mt5BridgeOnline ? 'ONLINE' : 'OFFLINE',
  });
}
