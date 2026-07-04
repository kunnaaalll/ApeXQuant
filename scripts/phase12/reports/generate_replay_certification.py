import psycopg2
import os
import datetime

def generate_report():
    print("Generating Replay Certification Report...")
    db_pass = os.environ.get('DB_PASSWORD', '')
    db_user = os.environ.get('POSTGRES_USER', 'apex')
    db_name = os.environ.get('POSTGRES_DB', 'apex_v3')
    
    try:
        conn = psycopg2.connect(host="localhost", database=db_name, user=db_user, password=db_pass)
        cur = conn.cursor()
        
        cur.execute("SELECT * FROM replay_hashes ORDER BY created_at DESC LIMIT 1;")
        row = cur.fetchone()
        
        if not row:
            print("No replay data found.")
            return
            
        report_dir = os.environ.get('PHASE12_REPORT_DIR', 'phase12_reports')
        os.makedirs(report_dir, exist_ok=True)
        report_path = os.path.join(report_dir, 'REPLAY_CERTIFICATION_REPORT.md')
        
        with open(report_path, 'w') as f:
            f.write(f"# APEX V3 — Replay Certification Report\n")
            f.write(f"**Generated:** {datetime.datetime.utcnow().isoformat()}Z\n\n")
            
            f.write("## Hashes\n")
            f.write(f"- Trade Count: {row[1]}\n")
            f.write(f"- Original Portfolio Hash: {row[2]}\n")
            f.write(f"- Original Positions Hash: {row[3]}\n")
            f.write(f"- Original Risk Hash: {row[4]}\n")
            f.write(f"- Original Events Hash: {row[5]}\n")
            f.write(f"- Replay Portfolio Hash: {row[6]}\n")
            f.write(f"- Replay Positions Hash: {row[7]}\n")
            f.write(f"- Replay Risk Hash: {row[8]}\n")
            f.write(f"- Replay Events Hash: {row[9]}\n\n")
            
            match_str = "✅ PASSED" if row[11] else "❌ FAILED"
            f.write(f"## Result\n")
            f.write(f"**{match_str}**\n")
            if not row[11]:
                f.write(f"Detail: {row[12]}\n")
                
        print(f"Report generated at {report_path}")
        
    except Exception as e:
        print(f"Error generating report: {e}")
    finally:
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    generate_report()
