#!/bin/bash
# Finish migrating remaining clnrm examples

set -e

echo "🔄 Finishing migration of remaining examples..."

# List remaining files that need migration
REMAINING=$(find examples -name "*.clnrm.toml" -type f | xargs grep -l "\[service\.\|\[\[scenario\]\]\|\[weaver\]\|\[\[expect\.span\]\]")

for file in $REMAINING; do
    echo "📝 Finishing migration: $file"
    
    # Update service to containers
    sed -i.bak 's/\[service\./[containers./g' "$file"
    
    # Update scenario to steps
    sed -i.bak 's/\[\[scenario\]\]/[[steps]]/g' "$file"
    
    # Update service references to container
    sed -i.bak 's/service = /container = /g' "$file"
    
    # Convert simple run commands to exec
    sed -i.bak 's/run = "echo /exec = ["echo", /g' "$file"
    sed -i.bak 's/"$/"]/g' "$file"
    
    # Remove weaver sections
    sed -i.bak '/^\[weaver\]$/,/^\[/d' "$file"
    sed -i.bak '/^\[weaver\./,/^\[/d' "$file"
    
    # Add basic OTEL config if missing
    if ! grep -q "\[otel\]" "$file"; then
        sed -i.bak '/^\[test\]$/,/description = ".*"/a\
\
# OTEL configuration\
[otel]\
service_name = "clnrm-example"' "$file"
    fi
    
    # Update expect.span to expect.otel
    if grep -q "\[expect.span\]" "$file"; then
        sed -i.bak 's/\[\[expect\.span\]\]/[expect.otel]\
spans = [/g' "$file"
        sed -i.bak 's/name = "\([^"]*\)"/{\n        name = "\1",\n        sample_count = { min = 1 }\n    }/g' "$file"
        echo "]" >> "$file"
    fi
    
    # Clean up
    rm -f "${file}.bak"
    
    echo "✅ Finished: $file"
done

echo "🎉 Migration complete!"
