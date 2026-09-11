#!/bin/bash

TEST_DIR="tests"
SAMPLES_DIR="samples"
EXECUTABLE="./mila"
LOG="test_log.log"
OUT_DIR="mtarget"

# results
passed_count=0
failed_count=0
total_tests=0

echo "Starting tests..."
echo "--------------------------------------------------"

# iterate over .in files
for infile in "$TEST_DIR"/*.in; do
    total_tests=$((total_tests + 1))

    # get names
    filename=$(basename "$infile")
    base_name=$(echo "$filename" | sed -n 's/^\(.*\)\.run[0-9]\+\.in$/\1/p')
    run_part=$(echo "$filename" | sed -n 's/^.*\.run\([0-9]\+\)\.in$/run\1/p')
    test_identifier="${base_name}.${run_part}"

    echo "--- Testing: $test_identifier ---"

    expected_output_file="$TEST_DIR/${test_identifier}.out"
    source_file="$SAMPLES_DIR/${base_name}.mila"

    # construct command
    command_to_run="$EXECUTABLE -cn $source_file"

    # temp file for my output
    actual_output_file=$(mktemp)

    # execute command
    $command_to_run >> $LOG 2>&1

    # check compilation succeeded
    if ! [[ -f "$OUT_DIR/$base_name" ]]; then
        echo "Could not compile $source_file"
         failed_count=$((failed_count + 1))
        continue
    fi

    # run program
    ./"$OUT_DIR/$base_name" < "$infile" > "$actual_output_file"

    # compare outputs
    if diff -q "$actual_output_file" "$expected_output_file" > /dev/null; then
        echo "Result: PASSED"
        passed_count=$((passed_count + 1))
    else
        echo "Result: FAILED (Output mismatch)"
        failed_count=$((failed_count + 1))

        # show diff
        echo "--- Diff ---"
        diff --label="Actual Output" --label="Expected Output" -u "$actual_output_file" "$expected_output_file"
        echo "-----------------------------------------"
    fi

    # cleanup
    rm -f "$actual_output_file"
    rm -f "$OUT_DIR/$base_name"

    echo

done

# final summary
echo "================== Test Summary =================="
echo "Total Tests Run:      $total_tests"
echo "PASSED:               $passed_count / $total_tests"
echo "FAILED:               $failed_count / $total_tests"
echo "=================================================="

# all tests
if [[ $failed_count -eq 0 && $total_executed -gt 0 ]]; then
    echo "ALL TESTS PASSED!"
    exit 0
fi
