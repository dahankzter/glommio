window.BENCHMARK_DATA = {
  "lastUpdate": 1771114733492,
  "repoUrl": "https://github.com/dahankzter/glommio",
  "entries": {
    "Timer Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "dahankzter@gmail.com",
            "name": "Henrik Ma Johansson",
            "username": "dahankzter"
          },
          "committer": {
            "email": "dahankzter@gmail.com",
            "name": "Henrik Ma Johansson",
            "username": "dahankzter"
          },
          "distinct": true,
          "id": "c433f417daccec40b415fc79e8100a19aa0d6343",
          "message": "fix: remove incompatible external-data-json-path config\n\nThe benchmark action cannot use both auto-push and\nexternal-data-json-path together. Removed the explicit path\nconfiguration to let the action manage data storage automatically.\n\nThe action still stores historical data in gh-pages and generates\ntrend charts. This fixes the error: \"auto-push must be false when\nexternal-data-json-path is set\".\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T01:03:22+01:00",
          "tree_id": "786e77721319f70fb986e4fff6e44728f80dae00",
          "url": "https://github.com/dahankzter/glommio/commit/c433f417daccec40b415fc79e8100a19aa0d6343"
        },
        "date": 1771114732693,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5182,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 81203,
            "range": "± 623",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 996519,
            "range": "± 11897",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11691367,
            "range": "± 190823",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15033,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 92688,
            "range": "± 308",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 793569,
            "range": "± 16997",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7859611,
            "range": "± 201233",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4362,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 95702,
            "range": "± 1494",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 827396,
            "range": "± 7841",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8174453,
            "range": "± 114288",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 170,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 174,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 315,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 304,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 113,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 112,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 112,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 129,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 115,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 114,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 122,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 129,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4432,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 47733,
            "range": "± 568",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 606560,
            "range": "± 48758",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6539078,
            "range": "± 198900",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11675,
            "range": "± 92",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58375,
            "range": "± 615",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546886,
            "range": "± 20644",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6825710,
            "range": "± 79704",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3974,
            "range": "± 364",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57218,
            "range": "± 68826",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 561655,
            "range": "± 24648",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6788802,
            "range": "± 97422",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3179,
            "range": "± 18005",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29402,
            "range": "± 62108",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 290601,
            "range": "± 128865",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 3035023,
            "range": "± 159206",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 12982,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89647,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1007137,
            "range": "± 23965",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 17512845,
            "range": "± 414672",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3777,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90857,
            "range": "± 1623",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1041743,
            "range": "± 5957",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17973740,
            "range": "± 338924",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39713,
            "range": "± 41291",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 297552,
            "range": "± 379320",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3450647,
            "range": "± 433934",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48104,
            "range": "± 135416",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 286559,
            "range": "± 1056866",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2839270,
            "range": "± 300235",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6435,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 149998,
            "range": "± 560",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1503662,
            "range": "± 10578",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 20113181,
            "range": "± 905144",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 106596,
            "range": "± 3804",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 395273,
            "range": "± 70121",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 4532617,
            "range": "± 465703",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 90730,
            "range": "± 2330",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 297741,
            "range": "± 771825",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2216569,
            "range": "± 120639",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 91276,
            "range": "± 2107",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 298066,
            "range": "± 732780",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2225135,
            "range": "± 114264",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}