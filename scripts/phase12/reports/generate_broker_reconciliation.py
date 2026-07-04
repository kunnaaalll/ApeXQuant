import psycopg2
import os
import datetime

def generate_report():
    print("Generating Broker Reconciliation Report...")
    db_pass = os.environ.get('DB_PASSWORD', '')
    db_user = os.environ.get('POSTGRES_USER', 'apex')
    db_name = os.environ.get('POSTGRES_DB', 'apex_v3')
    
    try:
        conn = psycopg2.connect(host="localhost", database=db_name, user=db_user, password=db_pass)
        cur = conn.cursor()
        
        cur.execute("SELECT * FROM reconciliation_log ORDER BY checked_at DESC LIMIT 1;")
        row = cur.fetchone()
        
        if not row:
            print("No reconciliation data found.")
            return
            
        report_dir = os.environ.get('PHASE12_REPORT_DIR', 'phase12_reports')
        os.makedirs(report_dir, exist_ok=True)
        report_path = os.path.join(report_dir, 'BROKER_RECONCILIATION_REPORT.md')
        
        with open(report_path, 'w') as f:
            f.write(f"# APEX V3 — Broker Reconciliation Report\n")
            f.write(f"**Generated:** {datetime.datetime.utcnow().isoformat()}Z\n\n")
            
            f.write("## Latest Metrics\n")
            f.write(f"- Broker Balance: {row[2]}\n")
            f.write(f"- Broker Equity: {row[3]}\n")
            f.write(f"- Broker Margin: {row[4]}\n")
            f.write(f"- Broker Free Margin: {row[5]}\n")
            f.write(f"- Broker Position Count: {row[6]}\n")
            f.write(f"- Broker Order Count: {row[7]}\n")
            f.write(f"- Broker Trade Count: {row[8]}\n")
            f.write(f"- Engine Position Count: {row[9]}\n")
            f.write(f"- Portfolio Position Count: {row[10]}\n")
            f.write(f"- DB Execution Event Count: {row[11]}\n\n")
            
            match_str = "✅ PASSED" if row[13] else "❌ FAILED (DRIFT DETECTED)"
            f.write(f"## Match Result\n")
            f.write(f"**{match_str}**\n")
            
        print(f"Report generated at {report_path}")
        
    except Exception as e:
        print(f"Error generating report: {e}")
    finally:
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    generate_report()
