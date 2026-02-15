window.BENCHMARK_DATA = {
  "lastUpdate": 1771116705269,
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
      },
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
          "id": "a960238de8ded6bf23ae53ab7fd00568a0f60cfc",
          "message": "perf: skip slow stress tests in coverage runs\n\nExclude test_shares_high_disparity from coverage runs as it takes\nover 60 seconds under coverage instrumentation (5-10x slowdown).\n\nCoverage is about code path exploration, not stress/endurance\ntesting. Skipping slow tests reduces CI coverage time from 10+\nminutes to ~3 minutes while maintaining full code path coverage.\n\nApplied to both CI workflow and Makefile coverage targets.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T01:29:22+01:00",
          "tree_id": "22070e173331a053d98aa1e2fdb8b69695cfdf21",
          "url": "https://github.com/dahankzter/glommio/commit/a960238de8ded6bf23ae53ab7fd00568a0f60cfc"
        },
        "date": 1771116705019,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5513,
            "range": "± 40",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 82789,
            "range": "± 1507",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 998517,
            "range": "± 11107",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11478675,
            "range": "± 155341",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15824,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 96962,
            "range": "± 1490",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 798614,
            "range": "± 11681",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7640261,
            "range": "± 21475",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4149,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 94203,
            "range": "± 1217",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 824223,
            "range": "± 10423",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8014956,
            "range": "± 138264",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 129,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 129,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 132,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 159,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 80,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 80,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 82,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 95,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 82,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 83,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 83,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 95,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4270,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 46945,
            "range": "± 785",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 589684,
            "range": "± 3419",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6154456,
            "range": "± 173063",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11796,
            "range": "± 150",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 59070,
            "range": "± 447824",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 551175,
            "range": "± 36879",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6734285,
            "range": "± 254074",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3632,
            "range": "± 372",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57221,
            "range": "± 416728",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 558651,
            "range": "± 38372",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6895348,
            "range": "± 251750",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3170,
            "range": "± 45568",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29338,
            "range": "± 102523",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 288659,
            "range": "± 46670",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2893406,
            "range": "± 62014",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13508,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89755,
            "range": "± 731",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1011685,
            "range": "± 7133",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 16677870,
            "range": "± 335078",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3781,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90553,
            "range": "± 776",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1044161,
            "range": "± 4717",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 16952502,
            "range": "± 132974",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39925,
            "range": "± 59840",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 295185,
            "range": "± 19542",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3146152,
            "range": "± 363877",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48873,
            "range": "± 297090",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 284351,
            "range": "± 58650",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2765940,
            "range": "± 190876",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6371,
            "range": "± 208",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 151329,
            "range": "± 1089",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1536159,
            "range": "± 5741",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 18792991,
            "range": "± 162104",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 103294,
            "range": "± 17227",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 299079,
            "range": "± 26815",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2132836,
            "range": "± 42940",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 91132,
            "range": "± 170251",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 297530,
            "range": "± 34628",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2221425,
            "range": "± 237782",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 89983,
            "range": "± 163623",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 294221,
            "range": "± 34929",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2121937,
            "range": "± 230037",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}