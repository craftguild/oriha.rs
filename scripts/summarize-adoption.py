#!/usr/bin/env python3
"""Summarize numeric fields in adoption_cost JSONL, including sample variability."""
import collections
import json
from pathlib import Path
import statistics
import sys


def summarize(path):
    groups = collections.defaultdict(list)
    for line in Path(path).read_text().splitlines():
        row = json.loads(line)
        groups[row["case"]].append(row)
    result = {}
    for case, rows in groups.items():
        fields = {}
        for key, value in rows[0].items():
            if key == "sample" or not isinstance(value, (int, float)):
                continue
            values = [row[key] for row in rows]
            median = statistics.median(values)
            q1, _, q3 = statistics.quantiles(values, n=4, method="inclusive")
            fields[key] = {"median": median, "min": min(values), "max": max(values),
                           "q1": q1, "q3": q3,
                           "mad": statistics.median(abs(v - median) for v in values)}
        result[case] = {"samples": len(rows), "outcome": rows[0]["outcome"], "fields": fields}
    return result


if __name__ == "__main__":
    print(json.dumps({path: summarize(path) for path in sys.argv[1:]}, indent=2))
