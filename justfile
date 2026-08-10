set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set dotenv-load

# 显示可用命令。
default:
    @just --list

# 列出 examples/ 下的实验主题及其 topic 摘要（一级目录）。
catalog:
    @if [ ! -d examples ]; then \
        echo '尚未创建 examples/ 目录。'; \
        exit 0; \
    fi; \
    find examples -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | LC_ALL=C sort | \
        while IFS= read -r topic; do \
            topic_file="examples/$topic/topic"; \
            if [ ! -f "$topic_file" ]; then \
                printf '\033[1;34m%s\033[0m: %s\n' "$topic" '缺少 topic 文件'; \
                continue; \
            fi; \
            IFS= read -r summary < "$topic_file" || summary=''; \
            printf '\033[1;34m%s\033[0m: %s\n' "$topic" "$summary"; \
        done

# 打印某个主题的 topic 内容。用法：just list <主题>
list topic:
    @case '{{ topic }}' in (*[!A-Za-z0-9_-]*|'') echo '主题名只能包含字母、数字、- 和 _。' >&2; exit 2;; esac; \
    topic_dir="examples/{{ topic }}"; \
    if [ ! -d "$topic_dir" ]; then echo "未找到主题：{{ topic }}" >&2; exit 2; fi; \
    topic_file="$topic_dir/topic"; \
    if [ ! -f "$topic_file" ]; then echo "主题 {{ topic }} 缺少 topic 文件：$topic_file" >&2; exit 2; fi; \
    cat "$topic_file"

# 运行指定 lab。用法：just run <主题> <三位编号> [传给示例的参数...]
run topic number *args:
    @case '{{ topic }}' in (*[!A-Za-z0-9_-]*|'') echo '主题名只能包含字母、数字、- 和 _。' >&2; exit 2;; esac; \
    case '{{ number }}' in ([0-9][0-9][0-9]) ;; (*) echo '编号必须是三位数字，例如 001。' >&2; exit 2;; esac; \
    topic_dir="examples/{{ topic }}"; \
    if [ ! -d "$topic_dir" ]; then echo "未找到主题：{{ topic }}" >&2; exit 2; fi; \
    matches=("$topic_dir"/lab-{{ number }}-*.rs); \
    if [ ! -e "${matches[0]}" ]; then echo "未找到示例：{{ topic }} / {{ number }}" >&2; exit 2; fi; \
    if [ "${#matches[@]}" -ne 1 ]; then echo "编号 {{ number }} 对应了多个示例，请修正文件名。" >&2; exit 2; fi; \
    file="${matches[0]##*/}"; target="{{ topic }}-${file%.rs}"; \
    echo "运行 $target（${matches[0]}）"; \
    cargo run --example "$target" -- {{ args }}
