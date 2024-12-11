#!/bin/bash

# 处理日志的函数
process_logs() {
  awk '
    BEGIN {
        sum = 0;       # 持续时间的总和
        count = 0;     # 持续时间的计数
        max = 0;       # 持续时间的最大值
        min = 0;       # 持续时间的最小值
        split("", durations);  # 用于存储每个持续时间以计算中位数
        batch_count = 0;  # 批次计数器
        print "╔══════════════════════════════════════════════╗";
        print "║   🚀 Podcast Crawler - Duration Analysis 📊  ║";
        print "╠══════════════════════════════════════════════╣";
    }

    /Batch processing completed/ {
        # 去除 ANSI 颜色代码
        gsub(/\x1b\[[0-9;]*m/, "", $0);

        # 提取批次信息
        batch_time = $1 " " $2;
        success_count = 0;
        failure_count = 0;
        total_duration = 0;
        batch_count++;

        # 提取成功计数
        if ($0 ~ /success_count=[0-9]+/) {
            split($0, success_arr, "success_count=");
            split(success_arr[2], success_val, " ");
            success_count = success_val[1];
        }

        # 提取失败计数
        if ($0 ~ /failure_count=[0-9]+/) {
            split($0, failure_arr, "failure_count=");
            split(failure_arr[2], failure_val, " ");
            failure_count = failure_val[1];
        }

        # 提取总持续时间
        if ($0 ~ /total_duration=[0-9.]+/) {
            split($0, total_arr, "total_duration=");
            split(total_arr[2], total_val, " ");
            total_duration = total_val[1];
        }

        printf "║ 📅 Batch %2d: %-25s       ║\n", batch_count, batch_time;
        printf "║ ✅ Success Count: %3d                        ║\n", success_count;
        printf "║ ❌ Failure Count: %3d                        ║\n", failure_count;
        printf "║ ⏱️ Total Duration: %7.3f seconds           ║\n", total_duration;
        print "╟──────────────────────────────────────────────╢";
    }

    /duration:/ {
        if ($0 ~ /duration: [0-9]+\.[0-9]+(s|ms)/) {
            split($0, fields, "duration: ");  # 先按 "duration: " 拆分
            split(fields[2], duration, " ");  # 再按空格拆分，取得 duration 值
            split(duration[1], parts, ",");  # 再按 "s" 或 "ms" 拆分

            # Check if ends with 'ms'
            if (parts[1] ~ /ms$/) {
                # Remove 'ms' and convert to seconds
                sub(/ms$/, "", parts[1]);
                parts[1] = parts[1] / 1000;
            }
            # Check if ends with 's'
            else if (parts[1] ~ /s$/) {
                # Remove 's'
                sub(/s$/, "", parts[1]);
            }
            value = parts[1];

            # 更新统计数据
            count++;
            durations[count] = value;
            sum += value;

            # 更新最大值和最小值
            if (count == 1) {
                max = value;
                min = value;
            } else {
                if (value > max) max = value;
                if (value < min) min = value;
            }
        }
    }

    END {
        # 处理统计数据
        if (count > 0) {
            avg = sum / count;  # 计算平均值

            # 计算中位数（冒泡排序）
            for (i = 1; i <= count; i++) {
                for (j = i + 1; j <= count; j++) {
                    if (durations[i] > durations[j]) {
                        tmp = durations[i];
                        durations[i] = durations[j];
                        durations[j] = tmp;
                    }
                }
            }

            # 计算中位数
            if (count % 2 == 0) {
                median = (durations[count/2] + durations[count/2 + 1]) / 2;
            } else {
                median = durations[int(count/2) + 1];
            }

            # 打印统计结果
            print "╠══════════════════════════════════════════════╣";
            print "║           📊 Overall Statistics 📈           ║";
            print "╟──────────────────────────────────────────────╢";
            printf "║ 🏁 Total Processed:     %4d tasks           ║\n", count;
            printf "║ 📈 Maximum Duration:   %7.3f seconds       ║\n", max;
            printf "║ 📉 Minimum Duration:   %7.3f seconds       ║\n", min;
            printf "║ 📊 Average Duration:   %7.3f seconds       ║\n", avg;
            printf "║ 📍 Median Duration:    %7.3f seconds       ║\n", median;
            print "╚══════════════════════════════════════════════╝";
        } else {
            # 如果没有数据，仍然打印一些信息
            print "╠══════════════════════════════════════════════╣";
            print "║           ❗ No Data Available ❗              ║";
            printf "║ 🔢 Total Batches Processed: %3d               ║\n", batch_count;
            print "╚══════════════════════════════════════════════╝";
        }
    }
  '
}

# 计算每个 Batch 的平均执行时间、最大时间、最小时间和中位数
calculate_average_duration() {
  local log_file="$1"

  # 检查日志文件是否存在
  if [ ! -f "$log_file" ]; then
    echo "错误：日志文件 $log_file 不存在"
    return 1
  fi

  # 检查日志文件是否可读
  if [ ! -r "$log_file" ]; then
    echo "错误：日志文件 $log_file 不可读"
    return 1
  fi

  # 打印日志文件的前几行
  echo "日志文件前5行:"
  head -n 5 "$log_file"

  # 打印日志文件大小
  echo "日志文件大小:"
  ls -l "$log_file"

  # 直接使用 process_logs1 处理日志文件
  cat "$log_file" | process_logs
}

# 运行 cargo 命令并处理日志的函数
run_cargo_and_process_logs() {
  local build_type="$1"

  echo -e "\033[1;36mRunning the cargo command...\033[0m"  # 青色加粗文本
  cargo run --quiet "$build_type" --package podcast_crawler --bin podcast_crawler 2>&1 | grep -v "warning:" | process_logs
}

# 统计平均执行时间的函数
run_average_duration_calculation() {
  LOG_DIR="logs"

  # 检查日志目录是否存在
  if [[ ! -d "$LOG_DIR" ]]; then
    echo "❌ Error: Logs directory not found at $LOG_DIR"
    return 1
  fi

  # 找到最近的日志文件
  LATEST_LOG=$(find "$LOG_DIR" -type f -name "app.log.*" -print0 | xargs -0 ls -t | head -n1)

  # 检查是否找到日志文件
  if [[ -z "$LATEST_LOG" ]]; then
    echo "❌ Error: No log files found in $LOG_DIR"
    return 1
  fi

  echo "📄 Using log file: $LATEST_LOG"
  calculate_average_duration "$LATEST_LOG"
}

# 默认构建类型为 dev
BUILD_TYPE=""

# 解析脚本参数以确定构建类型
while [[ "$#" -gt 0 ]]; do
  case $1 in
    --release)
      BUILD_TYPE="--release"
      shift
      ;;
    *)
      break
      ;;
  esac
done

# 根据输入参数执行相应的操作
case "$1" in
  run)
    run_cargo_and_process_logs "$BUILD_TYPE"
    ;;
  average)
    run_average_duration_calculation
    ;;
  *)
    echo "Usage: $0 {run|average} [--release]"
    ;;
esac
