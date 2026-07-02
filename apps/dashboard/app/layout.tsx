import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'APEX V3 Trading System Dashboard',
  description: 'Real-time monitoring and analytics for APEX V3 trading engines.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body style={{ 
        margin: 0, 
        padding: 0, 
        backgroundColor: '#030305',
        color: '#f4f4f5', 
        fontFamily: '"Inter", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
        minHeight: '100vh',
        overflowX: 'hidden',
        position: 'relative'
      }}>
        {/* Background glowing orbs for Liquid Glass effect */}
        <div style={{ position: 'fixed', top: '-15%', left: '-10%', width: '50vw', height: '50vw', background: 'radial-gradient(circle, rgba(59,130,246,0.15) 0%, transparent 70%)', filter: 'blur(80px)', zIndex: -1, pointerEvents: 'none' }} />
        <div style={{ position: 'fixed', bottom: '-20%', right: '-10%', width: '60vw', height: '60vw', background: 'radial-gradient(circle, rgba(16,185,129,0.12) 0%, transparent 70%)', filter: 'blur(100px)', zIndex: -1, pointerEvents: 'none' }} />
        <div style={{ position: 'fixed', top: '30%', left: '40%', width: '40vw', height: '40vw', background: 'radial-gradient(circle, rgba(139,92,246,0.1) 0%, transparent 70%)', filter: 'blur(90px)', zIndex: -1, pointerEvents: 'none' }} />
        
        {children}
      </body>
    </html>
  );
}
