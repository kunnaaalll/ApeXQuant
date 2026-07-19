import os

def fix_default(filepath, struct_name):
    with open(filepath, 'r') as f:
        content = f.read()
    if f"impl Default for {struct_name}" not in content:
        content += f"\nimpl Default for {struct_name} {{\n    fn default() -> Self {{\n        Self::new()\n    }}\n}}\n"
        with open(filepath, 'w') as f:
            f.write(content)

fix_default('services/performance-engine-rs/src/validation/benchmark.rs', 'PerformanceBenchmark')
fix_default('services/performance-engine-rs/src/validation/certification.rs', 'CertificationEngine')
fix_default('services/performance-engine-rs/src/validation/determinism.rs', 'DeterminismValidator')
fix_default('services/performance-engine-rs/src/validation/parity.rs', 'PerformanceParityValidator')
fix_default('services/performance-engine-rs/src/validation/replay.rs', 'ReplayValidator')
fix_default('services/performance-engine-rs/src/validation/stress.rs', 'StressSuite')
fix_default('services/performance-engine-rs/src/clustering/behavior_groups.rs', 'BehaviorGroups')
fix_default('services/performance-engine-rs/src/discovery/edge_discovery.rs', 'EdgeDiscovery')
fix_default('services/performance-engine-rs/src/memory/context_memory.rs', 'ContextMemory')

def fix_module_inception(filepath):
    if not os.path.exists(filepath):
        print(f"Not found: {filepath}")
        return
    with open(filepath, 'r') as f:
        lines = f.readlines()
    
    out = []
    in_mod_tests = False
    brace_count = 0
    for line in lines:
        if "mod tests {" in line:
            in_mod_tests = True
            brace_count = 1
            continue
        if in_mod_tests:
            if "{" in line:
                brace_count += line.count("{")
            if "}" in line:
                brace_count -= line.count("}")
                if brace_count == 0:
                    in_mod_tests = False
                    continue
            out.append(line)
        else:
            out.append(line)
            
    with open(filepath, 'w') as f:
        f.writelines(out)

fix_module_inception('services/performance-engine-rs/src/confidence/tests.rs')
fix_module_inception('services/performance-engine-rs/src/streaks/tests.rs')
fix_module_inception('services/performance-engine-rs/src/intelligence/tests.rs')
fix_module_inception('services/performance-engine-rs/src/drift/tests.rs')
fix_module_inception('services/performance-engine-rs/src/learning/tests.rs')

print("Fixed Defaults and Module Inceptions")
