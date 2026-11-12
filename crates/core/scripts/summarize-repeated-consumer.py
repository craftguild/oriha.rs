#!/usr/bin/env python3
"""Summarize preserved profile-repeated-consumer.py directories as JSON."""
import argparse
import json
from pathlib import Path
import statistics


def median(values):
    return statistics.median(values)


def summary(path):
    result = {}
    for schedule in ["serial", "two", "queued"]:
        alloc = sorted(path.glob(f"{schedule}-*-alloc.jsonl"))
        rss = sorted(path.glob(f"{schedule}-*-rss.jsonl"))
        detail = {"rss_peak_kib":[], "rss_final_200ms_median_kib":[],
                  "batch_peak_requested":[], "final_live_requested":[],
                  "cache_dropped_live_requested":[], "cache_retained_bytes":[],
                  "stages":{}}
        for file in rss:
            samples = json.loads(file.with_name(file.stem + "-rss.json").read_text())
            detail["rss_peak_kib"].append(max(s["rss_kib"] for s in samples))
            end = samples[-1]["elapsed_ns"]
            detail["rss_final_200ms_median_kib"].append(median(s["rss_kib"] for s in samples if s["elapsed_ns"] >= end-200_000_000))
        all_rows = [row for f in alloc for row in map(json.loads, f.read_text().splitlines())]
        timing_rows = [row["trace"] for f in rss for row in map(json.loads, f.read_text().splitlines()) if row["type"] == "body"]
        for file in alloc:
            rows = list(map(json.loads,file.read_text().splitlines()))
            detail["batch_peak_requested"].append(max(r["peak_requested"] for r in rows if r["type"] == "batch"))
            for row in rows:
                if row["type"] != "quiescence": continue
                if row["phase"] == "workers_joined": detail["final_live_requested"].append(row["live_requested"])
                if row["phase"] == "cache_dropped_workers_idle": detail["cache_dropped_live_requested"].append(row["live_requested"])
                if row["phase"] == "cache_retained_workers_idle": detail["cache_retained_bytes"].append(row["cache_bytes"])
        for cls in range(3):
            rows = [r["trace"] for r in all_rows if r["type"] == "body" and r["trace"]["class"] == cls]
            times = [r for r in timing_rows if r["class"] == cls]
            stages = {}
            for i,name in enumerate(["parse","regions","candidate_scan","annotation_plan","annotation_apply","render"]):
                stages[name] = {key: median(r["stages"][i][key] for r in rows) for key in ["live_delta","peak_extra","requested"]}
                stages[name]["ns_uninstrumented_median"] = median(r["stages"][i]["ns"] for r in times)
            routing = sorted(r["stages"][1]["ns"] + r["stages"][2]["ns"] for r in times)
            stages["routing_regions_plus_scan_ns"] = {"median":median(routing),"p95":routing[int(.95*(len(routing)-1))],"max":max(routing)}
            routing_with_parse = sorted(sum(r["stages"][i]["ns"] for i in [0,1,2]) for r in times)
            stages["routing_parse_regions_plus_scan_ns"] = {"median":median(routing_with_parse),"p95":routing_with_parse[int(.95*(len(routing_with_parse)-1))],"max":max(routing_with_parse)}
            detail["stages"][str(cls)] = stages
        result[schedule] = detail
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps({"before":summary(args.before),"after":summary(args.after)},indent=2)+"\n")
