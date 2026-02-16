window.BENCHMARK_DATA = {
  "lastUpdate": 1771246633023,
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
          "id": "45ce63a90cd7dd39038b3f1c1f58c1309e2e4e6f",
          "message": "chore: Apply rustfmt formatting to benches and task mod\n\nFormats import statements and reorders module declarations\nper rustfmt conventions.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T09:39:31+01:00",
          "tree_id": "90ca85fee5f82ed0faf88dd997830b55bd2a2f89",
          "url": "https://github.com/dahankzter/glommio/commit/45ce63a90cd7dd39038b3f1c1f58c1309e2e4e6f"
        },
        "date": 1771146104279,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5582,
            "range": "± 239",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 85639,
            "range": "± 686",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1033697,
            "range": "± 10279",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11197390,
            "range": "± 379170",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15459,
            "range": "± 207",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 95729,
            "range": "± 1011",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 797011,
            "range": "± 11334",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7746429,
            "range": "± 80560",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4121,
            "range": "± 282",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 93328,
            "range": "± 801",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 805769,
            "range": "± 24826",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8156965,
            "range": "± 92932",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 125,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 128,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 126,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 165,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 78,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 89,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 97,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 107,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 89,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 85,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 92,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 109,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4487,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 48527,
            "range": "± 1947",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 607676,
            "range": "± 3854",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6988908,
            "range": "± 214865",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11725,
            "range": "± 230",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58421,
            "range": "± 441655",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546730,
            "range": "± 41978",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6884228,
            "range": "± 300064",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3403,
            "range": "± 385",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 56877,
            "range": "± 456315",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 554979,
            "range": "± 38403",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 7054258,
            "range": "± 293844",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3216,
            "range": "± 46659",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29525,
            "range": "± 104480",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 290352,
            "range": "± 22694",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2929457,
            "range": "± 97680",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13468,
            "range": "± 154",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89554,
            "range": "± 943",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1013236,
            "range": "± 6310",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 16866230,
            "range": "± 396877",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 4273,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 99798,
            "range": "± 2518",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1124231,
            "range": "± 8787",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17344321,
            "range": "± 340303",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 40378,
            "range": "± 102444",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 296429,
            "range": "± 23944",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3503432,
            "range": "± 501913",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48298,
            "range": "± 230211",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 282579,
            "range": "± 64832",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2831163,
            "range": "± 214528",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6582,
            "range": "± 188",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 160684,
            "range": "± 1454",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1564909,
            "range": "± 5108",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 18852715,
            "range": "± 385488",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 106619,
            "range": "± 17288",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 303453,
            "range": "± 28829",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2163611,
            "range": "± 304204",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 90779,
            "range": "± 376926",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 297076,
            "range": "± 49075",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2219219,
            "range": "± 316614",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 91893,
            "range": "± 221543",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 295815,
            "range": "± 50670",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2223466,
            "range": "± 337711",
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
          "id": "a79766749786c7e3fc89ae11bcdb27e83be3fd2e",
          "message": "refactor: Improve arena allocator safety without performance cost\n\nEnhances safety of unsafe code in arena allocator while maintaining\nzero-overhead performance through compile-time optimization.\n\nSafety improvements:\n- Replace NonNull::new_unchecked() with checked new() variant\n- Add 7 debug assertions for bounds and corruption detection\n- Use checked_mul() to prevent integer overflow\n- Add comprehensive safety documentation explaining invariants\n\nAll improvements compile away in release builds (verified zero cost):\n- NonNull::new() null check optimized away\n- checked_mul() becomes regular multiplication\n- debug_assert!() compiled out entirely\n\nMaintains 26ns spawn latency with significantly improved safety\nguarantees in debug builds for early bug detection.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T09:52:40+01:00",
          "tree_id": "3b6b480d503fc16e42e1cdb84e080dd2d92bec8e",
          "url": "https://github.com/dahankzter/glommio/commit/a79766749786c7e3fc89ae11bcdb27e83be3fd2e"
        },
        "date": 1771146928733,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5531,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 85712,
            "range": "± 1142",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1023894,
            "range": "± 36346",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 10906341,
            "range": "± 225390",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15503,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 95364,
            "range": "± 1236",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 795818,
            "range": "± 12378",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7736505,
            "range": "± 31715",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4096,
            "range": "± 300",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 93684,
            "range": "± 1086",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 809289,
            "range": "± 10102",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8120172,
            "range": "± 21153",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 135,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 128,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 133,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 150,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 81,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 80,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 84,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 93,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 80,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 79,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 78,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 92,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4248,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 45691,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 578996,
            "range": "± 2497",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6046490,
            "range": "± 47543",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11667,
            "range": "± 23249",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58328,
            "range": "± 478009",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546025,
            "range": "± 36870",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6648987,
            "range": "± 273052",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3331,
            "range": "± 376",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 56955,
            "range": "± 462081",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 557324,
            "range": "± 37022",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6817174,
            "range": "± 286471",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3163,
            "range": "± 43837",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29235,
            "range": "± 111184",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 287428,
            "range": "± 17980",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2932058,
            "range": "± 49475",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13622,
            "range": "± 87",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89349,
            "range": "± 12957",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1002195,
            "range": "± 12393",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 17435325,
            "range": "± 726277",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3759,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90920,
            "range": "± 1946",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1029137,
            "range": "± 3865",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17431181,
            "range": "± 230237",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39847,
            "range": "± 26438",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 294290,
            "range": "± 1926",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3121359,
            "range": "± 33272",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48849,
            "range": "± 252590",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 280786,
            "range": "± 1484",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2754640,
            "range": "± 15539",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6342,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 150438,
            "range": "± 643",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1491995,
            "range": "± 7555",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 19419834,
            "range": "± 215492",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 102753,
            "range": "± 13767",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 297596,
            "range": "± 31433",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2422458,
            "range": "± 298391",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 89702,
            "range": "± 161376",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 282596,
            "range": "± 40158",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2212932,
            "range": "± 239345",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 89633,
            "range": "± 1521",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 293491,
            "range": "± 36786",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2217260,
            "range": "± 246274",
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
          "id": "8db71354808b64f6d237598a65ce2916b2e196e3",
          "message": "ci: Add Miri workflow for automatic UB detection\n\nAdds GitHub Actions workflow that automatically runs Miri when unsafe\ncode changes are detected in arena.rs or raw.rs.\n\nWorkflow triggers:\n- On PR when arena.rs or raw.rs are modified\n- On push to master when unsafe code files change\n- Manual trigger via workflow_dispatch\n\nFeatures:\n- Runs Miri on arena allocator tests (~30 seconds)\n- Posts success comment on PRs when Miri passes\n- Optional full library Miri check (manual trigger only)\n- Stricter Miri flags: strict-provenance, symbolic-alignment-check\n\nBenefits:\n- Catches undefined behavior in CI before merge\n- Automatic validation of unsafe code changes\n- No need to remember to run Miri locally\n- Fast feedback loop for unsafe code reviews\n\nThe workflow only runs when necessary (path-based triggers),\nminimizing CI resource usage while maximizing safety validation.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T16:10:56+01:00",
          "tree_id": "ace436f82408faa6399310cea428cdabf4cf2aab",
          "url": "https://github.com/dahankzter/glommio/commit/8db71354808b64f6d237598a65ce2916b2e196e3"
        },
        "date": 1771169934809,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5556,
            "range": "± 117",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 86284,
            "range": "± 762",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1030673,
            "range": "± 17706",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11527645,
            "range": "± 105652",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 16032,
            "range": "± 140",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 96009,
            "range": "± 1544",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 800466,
            "range": "± 19766",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7696170,
            "range": "± 69705",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4255,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 94006,
            "range": "± 763",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 824488,
            "range": "± 13262",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8636185,
            "range": "± 643421",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 125,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 117,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 131,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 148,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 82,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 82,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 84,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 97,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 78,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 78,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 84,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 98,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4306,
            "range": "± 77",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 47005,
            "range": "± 712",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 586311,
            "range": "± 3555",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 5901828,
            "range": "± 119600",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11802,
            "range": "± 23324",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58976,
            "range": "± 451894",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546723,
            "range": "± 18163",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 7714125,
            "range": "± 1100993",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3362,
            "range": "± 382",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 56593,
            "range": "± 434827",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 553712,
            "range": "± 39671",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6861298,
            "range": "± 264560",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3169,
            "range": "± 46064",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29175,
            "range": "± 967",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 287733,
            "range": "± 24609",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2851651,
            "range": "± 89202",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13490,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89717,
            "range": "± 1219",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1006897,
            "range": "± 6229",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 16656860,
            "range": "± 137591",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3752,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90362,
            "range": "± 928",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1034329,
            "range": "± 5028",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 16907803,
            "range": "± 106115",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39952,
            "range": "± 105670",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 296918,
            "range": "± 4448",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3561425,
            "range": "± 605875",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48359,
            "range": "± 337132",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 281695,
            "range": "± 60225",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2858819,
            "range": "± 278201",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6306,
            "range": "± 222",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 151673,
            "range": "± 1969",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1541389,
            "range": "± 7327",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 18977699,
            "range": "± 407195",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 107354,
            "range": "± 16381",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 304172,
            "range": "± 28817",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2132751,
            "range": "± 255348",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 89882,
            "range": "± 375812",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 294825,
            "range": "± 48595",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2209652,
            "range": "± 334251",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 89506,
            "range": "± 379207",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 294642,
            "range": "± 50133",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2216333,
            "range": "± 318703",
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
          "id": "c988a786938a38573b20b4f5643533fc24ff84db",
          "message": "fix: Prevent use-after-free when tasks outlive arena scope\n\nTasks could outlive the arena's lifetime when executors shut down,\ncausing \"free(): invalid pointer\" crashes. The previous logic checked\nTASK_ARENA.is_set() at deallocation time, but the arena might already\nbe dropped, leading to incorrect heap deallocation of arena pointers.\n\nSolution: Track allocation source via ARENA_ALLOCATED state bit (bit 5).\nSet during allocation, checked during deallocation. If arena-allocated\nbut arena is gone, skip deallocation since arena's memory block was\nalready freed when arena dropped.\n\nThis fixes test failures in CI where parallel tests (--test-threads=4)\nallowed tasks to outlive their arena scope during cleanup.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T17:50:51+01:00",
          "tree_id": "f1859998e72cd2fccf1fdd0f03e7dfc4a1fe8475",
          "url": "https://github.com/dahankzter/glommio/commit/c988a786938a38573b20b4f5643533fc24ff84db"
        },
        "date": 1771175634341,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5768,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 85827,
            "range": "± 3855",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1029755,
            "range": "± 5913",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11567344,
            "range": "± 77061",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15835,
            "range": "± 55",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 95429,
            "range": "± 1174",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 792771,
            "range": "± 13647",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7813824,
            "range": "± 23046",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4248,
            "range": "± 254",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 94321,
            "range": "± 1136",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 816129,
            "range": "± 11650",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8074828,
            "range": "± 23620",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 128,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 127,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 128,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 136,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 81,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 79,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 82,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 106,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 82,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 80,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 83,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 108,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4422,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 48152,
            "range": "± 480",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 607193,
            "range": "± 3374",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6111784,
            "range": "± 137154",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11679,
            "range": "± 22679",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58311,
            "range": "± 419846",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546383,
            "range": "± 37409",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6778300,
            "range": "± 247099",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3357,
            "range": "± 375",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57031,
            "range": "± 408647",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 556944,
            "range": "± 34364",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6788190,
            "range": "± 301647",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3179,
            "range": "± 45938",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29471,
            "range": "± 102903",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 287612,
            "range": "± 47875",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2860334,
            "range": "± 211781",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13452,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 89875,
            "range": "± 645",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1008775,
            "range": "± 4584",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 17298991,
            "range": "± 144322",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3792,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90243,
            "range": "± 4498",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1041864,
            "range": "± 3746",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17713981,
            "range": "± 262372",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39901,
            "range": "± 1074",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 296508,
            "range": "± 2062",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3150074,
            "range": "± 91202",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 47842,
            "range": "± 53998",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 280583,
            "range": "± 44346",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2765670,
            "range": "± 69819",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6460,
            "range": "± 319",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 150851,
            "range": "± 843",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1507052,
            "range": "± 4297",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 18990950,
            "range": "± 322088",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 104696,
            "range": "± 17136",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 302134,
            "range": "± 13020",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2173666,
            "range": "± 14077",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 90375,
            "range": "± 169248",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 296031,
            "range": "± 1865837",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2215493,
            "range": "± 229287",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 90606,
            "range": "± 177181",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 294719,
            "range": "± 35753",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2217022,
            "range": "± 249010",
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
          "id": "d5610fafa65ae9d24c707928dbc4386fab12c177",
          "message": "fix: Prevent use-after-free when tasks outlive arena scope\n\nTasks could outlive the arena's lifetime when executors shut down,\ncausing \"free(): invalid pointer\" crashes. The previous logic checked\nTASK_ARENA.is_set() at deallocation time, but the arena might already\nbe dropped, leading to incorrect heap deallocation of arena pointers.\n\nSolution: Track allocation source via ARENA_ALLOCATED state bit (bit 5).\nSet during allocation, checked during deallocation. If arena-allocated\nbut arena is gone, skip deallocation since arena's memory block was\nalready freed when arena dropped.\n\nThis fixes test failures in CI where parallel tests (--test-threads=4)\nallowed tasks to outlive their arena scope during cleanup.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-15T18:00:41+01:00",
          "tree_id": "ee352f01e6a3bbdd583747345db56da424c68fed",
          "url": "https://github.com/dahankzter/glommio/commit/d5610fafa65ae9d24c707928dbc4386fab12c177"
        },
        "date": 1771180845028,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5576,
            "range": "± 73",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 84737,
            "range": "± 2221",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1038286,
            "range": "± 6430",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11696536,
            "range": "± 252620",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15724,
            "range": "± 158",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 96241,
            "range": "± 765",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 807008,
            "range": "± 17012",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7775337,
            "range": "± 132205",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4205,
            "range": "± 403",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 95197,
            "range": "± 1158",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 832626,
            "range": "± 10952",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8212287,
            "range": "± 94792",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 131,
            "range": "± 26",
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
            "value": 133,
            "range": "± 28",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 151,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 84,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 86,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 84,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 100,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 82,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 81,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 84,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 102,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4485,
            "range": "± 195",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 48273,
            "range": "± 659",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 603669,
            "range": "± 3646",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6366412,
            "range": "± 202805",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11785,
            "range": "± 25516",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58930,
            "range": "± 483582",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 546415,
            "range": "± 39743",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6900316,
            "range": "± 277950",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3344,
            "range": "± 381",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57069,
            "range": "± 408081",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 558182,
            "range": "± 37216",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6862493,
            "range": "± 287104",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3200,
            "range": "± 45909",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29364,
            "range": "± 104148",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 288941,
            "range": "± 45476",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 2879362,
            "range": "± 85221",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13565,
            "range": "± 143",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 90034,
            "range": "± 1050",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1006034,
            "range": "± 4704",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 16736017,
            "range": "± 169280",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3767,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 90931,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1042814,
            "range": "± 7155",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17226492,
            "range": "± 230108",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39825,
            "range": "± 1014",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 295372,
            "range": "± 2652",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3191284,
            "range": "± 131643",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 49153,
            "range": "± 28140",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 282839,
            "range": "± 34730",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2802455,
            "range": "± 61421",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6357,
            "range": "± 253",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 152576,
            "range": "± 2228",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1536209,
            "range": "± 8307",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 19548837,
            "range": "± 161257",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 104854,
            "range": "± 5426",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 300659,
            "range": "± 1752",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2149335,
            "range": "± 192712",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 91813,
            "range": "± 2506",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 297709,
            "range": "± 39518",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2218180,
            "range": "± 232003",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 90844,
            "range": "± 176107",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 295449,
            "range": "± 42621",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2213481,
            "range": "± 251934",
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
          "id": "ac558d8c9c9c456512fd7b10f212620c2211b6a9",
          "message": "feat: add make targets for all test modules\n\nAdd granular test targets for each major module, making it easy to\ntest specific areas of the codebase without running the full suite.\n\nNew targets:\n- make test-arena        - Arena allocator tests (8 tests)\n- make test-channels     - Channel tests\n- make test-controllers  - Controller tests\n- make test-error        - Error handling tests (19 tests)\n- make test-executor     - Executor tests\n- make test-io           - I/O tests\n- make test-net          - Network tests\n- make test-sync         - Synchronization tests\n- make test-task         - Task tests (22 tests, includes arena integration)\n- make test-timer        - Timer tests\n\nBenefits:\n- Fast iteration on specific modules\n- Works perfectly on Lima (no resource limits)\n- Clear test organization\n- Easy to run related tests together\n\nUpdated help output to list all module targets in organized sections.\nAll targets tested and working on macOS/Lima.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-16T12:01:58+01:00",
          "tree_id": "723f827dda00478e40f03d9e2209af694dab9ba9",
          "url": "https://github.com/dahankzter/glommio/commit/ac558d8c9c9c456512fd7b10f212620c2211b6a9"
        },
        "date": 1771246207723,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5605,
            "range": "± 711",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 84756,
            "range": "± 1380",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1035048,
            "range": "± 6366",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 12745769,
            "range": "± 867717",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15901,
            "range": "± 445",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 96069,
            "range": "± 1656",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 799011,
            "range": "± 15108",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 8136084,
            "range": "± 424650",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4215,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 94711,
            "range": "± 1464",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 820170,
            "range": "± 9499",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8471572,
            "range": "± 565347",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 122,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 117,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 123,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 167,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 102,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 103,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 104,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 129,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 95,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 102,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 105,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 128,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4409,
            "range": "± 72",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 47424,
            "range": "± 989",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 596372,
            "range": "± 11971",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 7135382,
            "range": "± 363530",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11815,
            "range": "± 199",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 58967,
            "range": "± 434433",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 552624,
            "range": "± 43279",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6938398,
            "range": "± 379784",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3972,
            "range": "± 339",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57068,
            "range": "± 461035",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 559535,
            "range": "± 42026",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6851122,
            "range": "± 283071",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3172,
            "range": "± 45844",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 29539,
            "range": "± 105588",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 289641,
            "range": "± 50079",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 3125561,
            "range": "± 362847",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13631,
            "range": "± 223",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 90121,
            "range": "± 1786",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1022258,
            "range": "± 7044",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 16762950,
            "range": "± 388161",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3764,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 91576,
            "range": "± 2982",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1076132,
            "range": "± 24498",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17134819,
            "range": "± 464365",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 39283,
            "range": "± 63369",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 293663,
            "range": "± 20828",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3237380,
            "range": "± 461101",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48800,
            "range": "± 226712",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 285350,
            "range": "± 60379",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2901684,
            "range": "± 232596",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6370,
            "range": "± 227",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 151347,
            "range": "± 2449",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1503742,
            "range": "± 6328",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 20303341,
            "range": "± 787949",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 105825,
            "range": "± 19385",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 299126,
            "range": "± 5907",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2335418,
            "range": "± 298352",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 90524,
            "range": "± 165912",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 319498,
            "range": "± 1776379",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2224361,
            "range": "± 233448",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 90864,
            "range": "± 4723",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 296121,
            "range": "± 35538",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2220672,
            "range": "± 243404",
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
          "id": "a56c0e5f5641f8ed95cf5c35837ac8268b4b0d56",
          "message": "style: apply rustfmt formatting\n\nRun cargo fmt --all to apply standard Rust formatting.\n\nMinor changes:\n- Line wrapping adjustments in lib.rs prelude\n- Whitespace consistency in executor/mod.rs\n- Formatting cleanup in spawn_public.rs test\n- Minor adjustments in arena.rs and raw.rs\n\nNo functional changes, purely formatting.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-16T13:29:43+01:00",
          "tree_id": "162f922fc86577153b7fa7b3976c7cb12885c7a0",
          "url": "https://github.com/dahankzter/glommio/commit/a56c0e5f5641f8ed95cf5c35837ac8268b4b0d56"
        },
        "date": 1771246489446,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5909,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 75280,
            "range": "± 2556",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1080429,
            "range": "± 16636",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 12164874,
            "range": "± 128837",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 16895,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 99063,
            "range": "± 816",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 823114,
            "range": "± 18787",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7925217,
            "range": "± 62644",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4307,
            "range": "± 217",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 97250,
            "range": "± 1272",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 858223,
            "range": "± 16964",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8437501,
            "range": "± 96066",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 137,
            "range": "± 30",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 151,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 152,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 157,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 81,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 81,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 83,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 95,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 76,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 74,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/10000",
            "value": 76,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/100000",
            "value": 101,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4622,
            "range": "± 74",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 49347,
            "range": "± 425",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 642313,
            "range": "± 15972",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6785135,
            "range": "± 77232",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 12926,
            "range": "± 30815",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 65342,
            "range": "± 534770",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 609001,
            "range": "± 3496",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 7488866,
            "range": "± 406468",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3029,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 64069,
            "range": "± 482721",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 626465,
            "range": "± 46015",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 7668512,
            "range": "± 423706",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3437,
            "range": "± 55714",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 32562,
            "range": "± 116236",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 327734,
            "range": "± 32513",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 3355236,
            "range": "± 28806",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 14596,
            "range": "± 149",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 94446,
            "range": "± 2572",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1042330,
            "range": "± 6073",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 17765233,
            "range": "± 523715",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 4017,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 95842,
            "range": "± 104019",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1088288,
            "range": "± 30631",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 18494091,
            "range": "± 381249",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 42464,
            "range": "± 124528",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 315157,
            "range": "± 31154",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3764673,
            "range": "± 264035",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 52086,
            "range": "± 198744",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 299960,
            "range": "± 73465",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 3074147,
            "range": "± 256894",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6377,
            "range": "± 216",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 164660,
            "range": "± 3393",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1602909,
            "range": "± 5902",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 20886401,
            "range": "± 446284",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 104498,
            "range": "± 25045",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 309320,
            "range": "± 40485",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2346388,
            "range": "± 277262",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 97429,
            "range": "± 2786",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 318916,
            "range": "± 44117",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2441323,
            "range": "± 266111",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 96334,
            "range": "± 3733",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 324723,
            "range": "± 44863",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2461707,
            "range": "± 278997",
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
          "id": "67a98e615c9633e6db64d4db45aea7b35c55f7d1",
          "message": "fix: run fmt directly on macOS to avoid Lima filesystem issues\n\nMake 'make fmt' run cargo fmt directly on macOS instead of through Lima,\nsince Lima has read-only filesystem issues when trying to write formatted\nfiles back to macOS.\n\nChanges:\n- macOS: cargo fmt --all (direct, no Lima)\n- Linux: cargo fmt --all (through run_cargo as before)\n\nThis fixes the \"Read-only file system (os error 30)\" error when running\n'make fmt' on macOS.\n\nOther commands (lint, check, test) continue to run through Lima since\nthey are read-only operations.\n\nSigned-off-by: Henrik Ma Johansson <dahankzter@gmail.com>",
          "timestamp": "2026-02-16T13:33:32+01:00",
          "tree_id": "09ab67fd6b186a707414ba89f23abacc9c795b92",
          "url": "https://github.com/dahankzter/glommio/commit/67a98e615c9633e6db64d4db45aea7b35c55f7d1"
        },
        "date": 1771246632712,
        "tool": "cargo",
        "benches": [
          {
            "name": "timer_insert_btreemap/100",
            "value": 5636,
            "range": "± 163",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/1000",
            "value": 87615,
            "range": "± 2244",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/10000",
            "value": 1054180,
            "range": "± 10579",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_btreemap/100000",
            "value": 11530425,
            "range": "± 184833",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100",
            "value": 15772,
            "range": "± 102",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/1000",
            "value": 96690,
            "range": "± 1455",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/10000",
            "value": 793762,
            "range": "± 10211",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_timing_wheel/100000",
            "value": 7872413,
            "range": "± 33532",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100",
            "value": 4216,
            "range": "± 311",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/1000",
            "value": 96224,
            "range": "± 1191",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/10000",
            "value": 826922,
            "range": "± 50754",
            "unit": "ns/iter"
          },
          {
            "name": "timer_insert_staged_wheel/100000",
            "value": 8101060,
            "range": "± 69429",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/0",
            "value": 120,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/1000",
            "value": 121,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/10000",
            "value": 133,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_btreemap/100000",
            "value": 151,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/0",
            "value": 84,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/1000",
            "value": 83,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/10000",
            "value": 86,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_timing_wheel/100000",
            "value": 110,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/0",
            "value": 76,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "timer_single_insert_staged_wheel/1000",
            "value": 76,
            "range": "± 14",
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
            "value": 109,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100",
            "value": 4390,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/1000",
            "value": 47446,
            "range": "± 981",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/10000",
            "value": 607637,
            "range": "± 5831",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_btreemap/100000",
            "value": 6385240,
            "range": "± 140391",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100",
            "value": 11824,
            "range": "± 25885",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/1000",
            "value": 59003,
            "range": "± 446132",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/10000",
            "value": 547701,
            "range": "± 42734",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_timing_wheel/100000",
            "value": 6755502,
            "range": "± 284975",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100",
            "value": 3177,
            "range": "± 366",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/1000",
            "value": 57207,
            "range": "± 430708",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/10000",
            "value": 558530,
            "range": "± 43730",
            "unit": "ns/iter"
          },
          {
            "name": "timer_remove_staged_wheel/100000",
            "value": 6762226,
            "range": "± 385302",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100",
            "value": 3272,
            "range": "± 50673",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/1000",
            "value": 30512,
            "range": "± 107078",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/10000",
            "value": 297445,
            "range": "± 25186",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_btreemap/100000",
            "value": 3083201,
            "range": "± 81357",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100",
            "value": 13612,
            "range": "± 126",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/1000",
            "value": 90744,
            "range": "± 929",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/10000",
            "value": 1007648,
            "range": "± 14539",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_timing_wheel/100000",
            "value": 17623826,
            "range": "± 281709",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100",
            "value": 3800,
            "range": "± 110",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/1000",
            "value": 93165,
            "range": "± 103900",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/10000",
            "value": 1053379,
            "range": "± 6075",
            "unit": "ns/iter"
          },
          {
            "name": "timer_process_expired_staged_wheel/100000",
            "value": 17424198,
            "range": "± 193278",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/1000",
            "value": 40517,
            "range": "± 93194",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/10000",
            "value": 301064,
            "range": "± 4890",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_btreemap/100000",
            "value": 3244378,
            "range": "± 149608",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/1000",
            "value": 48971,
            "range": "± 335942",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/10000",
            "value": 285331,
            "range": "± 62870",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_timing_wheel/100000",
            "value": 2767669,
            "range": "± 210759",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100",
            "value": 6616,
            "range": "± 295",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/1000",
            "value": 155489,
            "range": "± 1563",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/10000",
            "value": 1563711,
            "range": "± 10193",
            "unit": "ns/iter"
          },
          {
            "name": "timer_mixed_workload_staged_wheel/100000",
            "value": 19474106,
            "range": "± 219254",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/1000",
            "value": 106361,
            "range": "± 13782",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/10000",
            "value": 311313,
            "range": "± 33529",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_btreemap/100000",
            "value": 2230833,
            "range": "± 56094",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/1000",
            "value": 90574,
            "range": "± 1682",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/10000",
            "value": 294506,
            "range": "± 36577",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_timing_wheel/100000",
            "value": 2226635,
            "range": "± 262731",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/1000",
            "value": 91497,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/10000",
            "value": 297067,
            "range": "± 37974",
            "unit": "ns/iter"
          },
          {
            "name": "timer_churn_staged_wheel/100000",
            "value": 2222316,
            "range": "± 245251",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}