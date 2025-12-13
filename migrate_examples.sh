#!/bin/bash
# Migrate all clnrm examples from v1.x to v2.0.0 format

set -e

echo "🔄 Migrating clnrm examples to v2.0.0 format..."

# Find all .clnrm.toml files
find examples -name "*.clnrm.toml" -type f | while read -r file; do
    echo "📝 Migrating: $file"
    
    # Create backup
    cp "$file" "${file}.backup"
    
    # Apply migrations
    # 1. Update metadata section
    sed -i.bak 's/\[test\.metadata\]/[test]/g' "$file"
    
    # 2. Update service sections to containers
    sed -i.bak 's/\[service\./[containers./g' "$file"
    
    # 3. Update scenario to steps
    sed -i.bak 's/\[\[scenario\]\]/[[steps]]/g' "$file"
    
    # 4. Update service references to container
    sed -i.bak 's/service = /container = /g' "$file"
    
    # 5. Convert run commands to exec arrays (simple cases)
    # Handle simple echo commands
    sed -i.bak 's/run = "echo '\''\([^'\'']*\)'\''"/exec = ["echo", "\1"]/g' "$file"
    
    # Handle other simple commands
    sed -i.bak 's/run = "\([^"]*\)"/exec = ["\1"]/g' "$file"
    
    # 6. Remove/replace weaver sections with OTEL
    if grep -q "\[weaver\]" "$file"; then
        # Add OTEL section after test section
        sed -i.bak '/^\[test\]$/,/description = ".*"/a\
\
# OTEL configuration for telemetry validation\
[otel]\
service_name = "clnrm-example"' "$file"
        
        # Remove weaver sections
        sed -i.bak '/^\[weaver\]$/,/^\[/d' "$file"
        sed -i.bak '/^\[weaver\./,/^\[/d' "$file"
    fi
    
    # 7. Update expectations from [[expect.span]] to [expect.otel]
    if grep -q "\[expect.span\]" "$file"; then
        sed -i.bak 's/\[\[expect\.span\]\]/[expect.otel]\
spans = [/g' "$file"
        
        # Close the spans array and add sample_count
        sed -i.bak 's/attrs = { all = { "\([^"]*\)" = "\([^"]*\)" } }/{\n        name = "\1",\n        sample_count = { min = 1 }\n    }/' "$file"
        
        # Add closing bracket for spans array
        echo "]" >> "$file"
    fi
    
    # Clean up temporary files
    rm -f "${file}.bak"
    
    echo "✅ Migrated: $file"
done

echo "🎉 Migration complete! All examples updated to v2.0.0 format."
echo "📋 Summary of changes:"
echo "  - [test.metadata] → [test]"
echo "  - [service.*] → [containers.*]" 
echo "  - [[scenario]] → [[steps]]"
echo "  - service = → container ="
echo "  - run = \"...\" → exec = [\"...\"]"
echo "  - [weaver] → [otel]"
echo "  - [[expect.span]] → [expect.otel]"
