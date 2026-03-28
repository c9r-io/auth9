#!/usr/bin/env zsh
# Auth9 清理脚本
#
# 本脚本用于从 Kubernetes 集群中清理 Auth9 资源。
#
# 用法:
#   ./cleanup.sh [选项]
#
# 选项:
#   --namespace NS       使用其他命名空间（默认: auth9）
#   --dry-run            仅显示将要删除的内容，不实际执行
#   --legacy-keycloak-only  仅清理旧 Keycloak 资源
#   --workloads-only     仅清理工作负载，保留配置和数据
#   --full               执行全量清理（默认）
#   --reset-db           执行数据库重置
#   --delete-secrets     删除 Secrets
#   --delete-pvc         删除 PVC
#   --delete-namespace   删除命名空间

set -e

# Configuration
NAMESPACE="${NAMESPACE:-auth9}"
DRY_RUN=""
MODE="full"
RESET_DB=""
DELETE_SECRETS=""
DELETE_PVCS=""
DELETE_NAMESPACE=""

typeset -a LEGACY_DEPLOYMENTS=(keycloak)
typeset -a LEGACY_STATEFULSETS=(keycloak-postgres)
typeset -a LEGACY_HPAS=(keycloak)
typeset -a LEGACY_SERVICES=(keycloak keycloak-headless keycloak-public keycloak-postgres)
typeset -a LEGACY_CONFIGMAPS=(keycloak-config keycloak-nginx-gw)
typeset -a LEGACY_SECRETS=(keycloak-secrets)
typeset -a LEGACY_PVCS=(postgres-data-keycloak-postgres-0)

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

print_header() {
    local title="$1"
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
    printf "${BLUE}║${NC} %-42s ${BLUE}║${NC}\n" "$title"
    echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
    echo ""
}

print_success() {
    echo -e "  ${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "  ${RED}✗${NC} $1"
}

print_warning() {
    echo -e "  ${YELLOW}⚠${NC} $1"
}

print_info() {
    echo -e "  ${CYAN}ℹ${NC} $1"
}

print_progress() {
    local step="$1"
    local message="$2"
    echo ""
    echo -e "${GREEN}[$step]${NC} ${BOLD}$message${NC}"
}

confirm_action() {
    local message="$1"
    local response

    while true; do
        read "response?$message [y/N]: "
        case "$response" in
            [Yy]* ) return 0 ;;
            [Nn]* | "" ) return 1 ;;
            * ) echo "请回答 yes 或 no。" ;;
        esac
    done
}

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --namespace)
                NAMESPACE="$2"
                shift 2
                ;;
            --dry-run)
                DRY_RUN="true"
                shift
                ;;
            --legacy-keycloak-only)
                MODE="legacy-keycloak-only"
                shift
                ;;
            --workloads-only)
                MODE="workloads-only"
                shift
                ;;
            --full)
                MODE="full"
                shift
                ;;
            --reset-db)
                RESET_DB="true"
                shift
                ;;
            --delete-secrets)
                DELETE_SECRETS="true"
                shift
                ;;
            --delete-pvc)
                DELETE_PVCS="true"
                shift
                ;;
            --delete-namespace)
                DELETE_NAMESPACE="true"
                shift
                ;;
            -h|--help)
                echo "用法: $0 [选项]"
                echo ""
                echo "选项:"
                echo "  --namespace NS       使用其他命名空间（默认: auth9）"
                echo "  --dry-run            仅显示将要删除的内容，不实际执行"
                echo "  --legacy-keycloak-only  仅清理旧 Keycloak 资源"
                echo "  --workloads-only     仅清理工作负载，保留配置和数据"
                echo "  --full               执行全量清理（默认）"
                echo "  --reset-db           执行数据库重置"
                echo "  --delete-secrets     删除 Secrets"
                echo "  --delete-pvc         删除 PVC"
                echo "  --delete-namespace   删除命名空间"
                echo "  -h, --help           显示帮助信息"
                exit 0
                ;;
            *)
                echo -e "${RED}未知选项: $1${NC}"
                echo "使用 --help 查看用法信息"
                exit 1
                ;;
        esac
    done
}

check_namespace() {
    if ! kubectl get namespace "$NAMESPACE" &>/dev/null; then
        print_warning "命名空间 '$NAMESPACE' 不存在"
        exit 0
    fi
}

show_resources() {
    echo -e "${BOLD}命名空间 '$NAMESPACE' 中的当前资源:${NC}"
    echo ""

    echo -e "${YELLOW}Deployments:${NC}"
    kubectl get deployments -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""

    echo -e "${YELLOW}StatefulSets:${NC}"
    kubectl get statefulsets -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""

    echo -e "${YELLOW}Jobs:${NC}"
    kubectl get jobs -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""

    echo -e "${YELLOW}HorizontalPodAutoscalers:${NC}"
    kubectl get hpa -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""

    echo -e "${YELLOW}Services:${NC}"
    kubectl get services -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""

    echo -e "${YELLOW}Secrets:${NC}"
    kubectl get secrets -n "$NAMESPACE" --no-headers 2>/dev/null | grep -v "default-token\|service-account" || echo "  （无）"
    echo ""

    echo -e "${YELLOW}ConfigMaps:${NC}"
    kubectl get configmaps -n "$NAMESPACE" --no-headers 2>/dev/null | grep -v "kube-root-ca" || echo "  （无）"
    echo ""

    echo -e "${YELLOW}PVCs（数据库数据）:${NC}"
    kubectl get pvc -n "$NAMESPACE" --no-headers 2>/dev/null || echo "  （无）"
    echo ""
}

print_mode_summary() {
    case "$MODE" in
        legacy-keycloak-only)
            print_info "模式: 仅清理旧 Keycloak 资源"
            [ -n "$DELETE_PVCS" ] && print_info "附加选项: 删除旧 Keycloak PVC"
            ;;
        workloads-only)
            print_info "模式: 仅清理工作负载（保留 ConfigMap / Secret / PVC / Namespace）"
            [ -n "$RESET_DB" ] && print_info "附加选项: 重置数据库"
            [ -n "$DELETE_SECRETS" ] && print_info "附加选项: 删除 Secrets"
            [ -n "$DELETE_PVCS" ] && print_info "附加选项: 删除 PVC"
            [ -n "$DELETE_NAMESPACE" ] && print_info "附加选项: 删除命名空间"
            ;;
        *)
            print_info "模式: 全量清理"
            ;;
    esac
}

count_resources() {
    local resources="$1"
    echo "$resources" | sed '/^$/d' | wc -l | tr -d ' '
}

named_resource_list() {
    local kind="$1"
    shift
    local resources=""

    for name in "$@"; do
        if kubectl get "$kind" "$name" -n "$NAMESPACE" &>/dev/null; then
            resources+="${kind}/${name}"$'\n'
        fi
    done

    printf '%s' "$resources"
}

list_user_secrets() {
    kubectl get secrets -n "$NAMESPACE" -o name 2>/dev/null | grep -v "default-token\|service-account" || true
}

list_configmaps() {
    kubectl get configmaps -n "$NAMESPACE" -o name 2>/dev/null | grep -v "kube-root-ca" || true
}

list_pvcs() {
    kubectl get pvc -n "$NAMESPACE" -o name 2>/dev/null || true
}

delete_resource_list() {
    local description="$1"
    local resources="$2"
    local count
    count=$(count_resources "$resources")

    if [ "$count" -eq 0 ]; then
        print_info "没有 ${description} 需要删除"
        return
    fi

    print_info "正在删除 $count 个${description}..."
    if [ -n "$DRY_RUN" ]; then
        echo "$resources" | sed '/^$/d'
        return
    fi

    while IFS= read -r resource; do
        [ -n "$resource" ] || continue
        kubectl delete "$resource" -n "$NAMESPACE" --ignore-not-found=true
    done <<< "$resources"

    print_success "${description} 已删除"
}

show_legacy_keycloak_resources() {
    local resources=""
    local part

    for part in \
        "$(named_resource_list deployment "${LEGACY_DEPLOYMENTS[@]}")" \
        "$(named_resource_list statefulset "${LEGACY_STATEFULSETS[@]}")" \
        "$(named_resource_list hpa "${LEGACY_HPAS[@]}")" \
        "$(named_resource_list service "${LEGACY_SERVICES[@]}")" \
        "$(named_resource_list configmap "${LEGACY_CONFIGMAPS[@]}")" \
        "$(named_resource_list secret "${LEGACY_SECRETS[@]}")" \
        "$(named_resource_list pvc "${LEGACY_PVCS[@]}")"
    do
        [ -n "$part" ] || continue
        resources+="$part"$'\n'
    done

    if [ "$(count_resources "$resources")" -eq 0 ]; then
        print_info "未检测到旧 Keycloak 资源"
        return
    fi

    echo -e "${BOLD}旧 Keycloak 资源:${NC}"
    echo "$resources" | sed '/^$/d' | sed 's/^/  /'
    echo ""
}

delete_jobs() {
    local job_count=$(kubectl get jobs -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$job_count" -eq 0 ]; then
        print_info "没有 Job 需要删除"
        return
    fi

    print_info "正在删除 $job_count 个 Job..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get jobs -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete jobs --all -n "$NAMESPACE" --ignore-not-found=true
        print_success "Jobs 已删除"
    fi
}

delete_deployments() {
    local deploy_count=$(kubectl get deployments -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$deploy_count" -eq 0 ]; then
        print_info "没有 Deployment 需要删除"
        return
    fi

    print_info "正在删除 $deploy_count 个 Deployment..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get deployments -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete deployments --all -n "$NAMESPACE" --ignore-not-found=true
        print_success "Deployments 已删除"
    fi
}

delete_statefulsets() {
    local sts_count=$(kubectl get statefulsets -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$sts_count" -eq 0 ]; then
        print_info "没有 StatefulSet 需要删除"
        return
    fi

    print_info "正在删除 $sts_count 个 StatefulSet..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get statefulsets -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete statefulsets --all -n "$NAMESPACE" --ignore-not-found=true
        print_success "StatefulSets 已删除"
    fi
}

delete_services() {
    local svc_count=$(kubectl get services -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$svc_count" -eq 0 ]; then
        print_info "没有 Service 需要删除"
        return
    fi

    print_info "正在删除 $svc_count 个 Service..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get services -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete services --all -n "$NAMESPACE" --ignore-not-found=true
        print_success "Services 已删除"
    fi
}

delete_hpas() {
    local hpa_count=$(kubectl get hpa -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$hpa_count" -eq 0 ]; then
        print_info "没有 HPA 需要删除"
        return
    fi

    print_info "正在删除 $hpa_count 个 HPA..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get hpa -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete hpa --all -n "$NAMESPACE" --ignore-not-found=true
        print_success "HPAs 已删除"
    fi
}

delete_configmaps() {
    delete_resource_list "ConfigMap" "$(list_configmaps)"
}

delete_observability_resources() {
    # Clean up Prometheus Operator CRDs and observability resources
    local has_resources=false

    if kubectl get servicemonitor auth9-core -n "$NAMESPACE" &>/dev/null 2>&1; then
        has_resources=true
    fi
    if kubectl get prometheusrule auth9-core -n "$NAMESPACE" &>/dev/null 2>&1; then
        has_resources=true
    fi

    if [ "$has_resources" = "false" ]; then
        print_info "没有可观测性资源需要删除"
        return
    fi

    print_info "正在删除可观测性资源..."
    if [ -n "$DRY_RUN" ]; then
        kubectl get servicemonitor -n "$NAMESPACE" -o name 2>/dev/null || true
        kubectl get prometheusrule -n "$NAMESPACE" -o name 2>/dev/null || true
    else
        kubectl delete servicemonitor auth9-core -n "$NAMESPACE" --ignore-not-found=true 2>/dev/null || true
        kubectl delete prometheusrule auth9-core -n "$NAMESPACE" --ignore-not-found=true 2>/dev/null || true
        print_success "可观测性资源已删除"
    fi
}

reset_tidb_database() {
    local skip_confirm="${1:-}"
    local step_label="${2:-7/10}"
    print_progress "$step_label" "重置 TiDB 数据库"

    echo ""
    print_warning "此操作将删除 auth9 数据库中的所有数据！"
    echo ""

    if [ -n "$DRY_RUN" ]; then
        print_info "[预演] 将询问是否重置数据库"
        return 0
    fi

    if [ -z "$skip_confirm" ] && ! confirm_action "  确定要重置数据库吗？"; then
        print_info "跳过数据库重置"
        return 1
    fi

    # 检查 secret 是否存在（需要从中获取 DATABASE_URL）
    if ! kubectl get secret auth9-secrets -n "$NAMESPACE" &>/dev/null; then
        print_error "auth9-secrets 不存在，无法获取数据库连接信息"
        return 1
    fi

    print_info "正在运行数据库重置..."

    # 删除旧的 reset job（如果存在）
    kubectl delete job auth9-reset -n "$NAMESPACE" --ignore-not-found=true 2>/dev/null

    # 创建 reset job
    cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: auth9-reset
  namespace: $NAMESPACE
spec:
  ttlSecondsAfterFinished: 60
  template:
    spec:
      restartPolicy: Never
      containers:
        - name: auth9-reset
          image: ghcr.io/c9r-io/auth9-core:latest
          command: ["auth9-core", "reset"]
          envFrom:
            - secretRef:
                name: auth9-secrets
EOF

    # 等待 Job 完成
    print_info "等待数据库重置完成（超时: 120秒）..."
    if kubectl wait --for=condition=complete job/auth9-reset -n "$NAMESPACE" --timeout=120s 2>/dev/null; then
        # 显示日志
        echo ""
        kubectl logs job/auth9-reset -n "$NAMESPACE" 2>/dev/null || true
        echo ""

        # 清理 job
        kubectl delete job auth9-reset -n "$NAMESPACE" --ignore-not-found=true 2>/dev/null
        print_success "数据库已重置"

        return 0
    else
        print_error "数据库重置失败或超时"
        echo ""
        echo "  查看日志: kubectl logs job/auth9-reset -n $NAMESPACE"
        kubectl logs job/auth9-reset -n "$NAMESPACE" --tail=10 2>/dev/null || true
        return 1
    fi
}

delete_user_secrets() {
    delete_resource_list "Secret" "$(list_user_secrets)"
}

interactive_delete_secrets() {
    local skip_confirm="${1:-}"
    local step_label="${2:-8/10}"
    print_progress "$step_label" "Secrets"

    echo ""
    echo -e "  ${YELLOW}当前密钥:${NC}"
    list_user_secrets | while read line; do
        echo "    $line"
    done || echo "    （无）"
    echo ""

    print_warning "密钥包含敏感数据:"
    echo "    - DATABASE_URL（数据库连接字符串）"
    echo "    - REDIS_URL"
    echo "    - JWT_SECRET、JWT_PRIVATE_KEY、JWT_PUBLIC_KEY"
    echo "    - SESSION_SECRET、SETTINGS_ENCRYPTION_KEY、PASSWORD_RESET_HMAC_KEY"
    echo "    - GRPC_API_KEYS"
    echo ""

    if [ -n "$DRY_RUN" ]; then
        print_info "[预演] 将询问是否删除密钥"
        return
    fi

    if [ -n "$skip_confirm" ] || confirm_action "  删除剩余密钥？（下次部署需要重新配置）"; then
        delete_user_secrets
    else
        print_info "保留密钥"
    fi
}

delete_all_pvcs() {
    delete_resource_list "PVC" "$(list_pvcs)"
}

interactive_delete_pvcs() {
    local skip_confirm="${1:-}"
    local step_label="${2:-9/10}"
    print_progress "$step_label" "持久卷声明（数据库数据）"

    local pvc_count=$(kubectl get pvc -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | tr -d ' ')
    if [ "$pvc_count" -eq 0 ]; then
        print_info "没有 PVC 需要删除"
        return
    fi

    echo ""
    echo -e "  ${YELLOW}当前 PVCs:${NC}"
    kubectl get pvc -n "$NAMESPACE" --no-headers 2>/dev/null | while read line; do
        echo "    $line"
    done
    echo ""

    print_warning "PVCs 包含数据库数据"
    echo ""
    echo -e "  ${RED}警告: 删除 PVCs 将永久销毁所有数据库数据！${NC}"
    echo ""

    if [ -n "$DRY_RUN" ]; then
        print_info "[预演] 将询问是否删除 PVCs"
        return
    fi

    if [ -n "$skip_confirm" ] || confirm_action "  删除 PVCs？（这将销毁所有数据库数据）"; then
        delete_all_pvcs
    else
        print_info "保留 PVCs（数据库数据已保留）"
    fi
}

delete_namespace_now() {
    if [ -n "$DRY_RUN" ]; then
        print_info "[预演] 将删除命名空间 '$NAMESPACE'"
        return
    fi

    print_info "正在删除命名空间..."
    kubectl delete namespace "$NAMESPACE" --ignore-not-found=true
    print_success "命名空间已删除"
}

interactive_delete_namespace() {
    local skip_confirm="${1:-}"
    local step_label="${2:-10/10}"
    print_progress "$step_label" "命名空间"

    echo ""
    print_info "删除命名空间将移除所有剩余资源"
    echo ""

    if [ -n "$DRY_RUN" ]; then
        print_info "[预演] 将删除命名空间 '$NAMESPACE'"
        return
    fi

    if [ -n "$skip_confirm" ] || confirm_action "  删除命名空间 '$NAMESPACE'？"; then
        delete_namespace_now
    else
        print_info "保留命名空间"
    fi
}

run_full_cleanup() {
    if ! confirm_action "开始清理？"; then
        print_info "清理已取消"
        exit 0
    fi

    print_header "正在清理资源"

    print_progress "1/10" "Jobs"
    delete_jobs

    print_progress "2/10" "Deployments"
    delete_deployments

    print_progress "3/10" "StatefulSets"
    delete_statefulsets

    print_progress "4/10" "HorizontalPodAutoscalers"
    delete_hpas

    print_progress "5/10" "Services 和 ConfigMaps"
    delete_services
    delete_configmaps

    print_progress "6/10" "可观测性资源"
    delete_observability_resources

    if [ -n "$RESET_DB" ]; then
        reset_tidb_database "skip-confirm" "7/10"
    else
        reset_tidb_database "" "7/10"
    fi

    if [ -n "$DELETE_SECRETS" ]; then
        interactive_delete_secrets "skip-confirm" "8/10"
    else
        interactive_delete_secrets "" "8/10"
    fi

    if [ -n "$DELETE_PVCS" ]; then
        interactive_delete_pvcs "skip-confirm" "9/10"
    else
        interactive_delete_pvcs "" "9/10"
    fi

    if [ -n "$DELETE_NAMESPACE" ]; then
        interactive_delete_namespace "skip-confirm" "10/10"
    else
        interactive_delete_namespace "" "10/10"
    fi
}

run_legacy_keycloak_cleanup() {
    show_legacy_keycloak_resources

    if ! confirm_action "开始清理旧 Keycloak 资源？"; then
        print_info "清理已取消"
        exit 0
    fi

    print_header "正在清理旧 Keycloak 资源"

    print_progress "1/4" "Deployments / StatefulSets / HPA"
    delete_resource_list "Legacy Deployment" "$(named_resource_list deployment "${LEGACY_DEPLOYMENTS[@]}")"
    delete_resource_list "Legacy StatefulSet" "$(named_resource_list statefulset "${LEGACY_STATEFULSETS[@]}")"
    delete_resource_list "Legacy HPA" "$(named_resource_list hpa "${LEGACY_HPAS[@]}")"

    print_progress "2/4" "Services"
    delete_resource_list "Legacy Service" "$(named_resource_list service "${LEGACY_SERVICES[@]}")"

    print_progress "3/4" "ConfigMaps / Secrets"
    delete_resource_list "Legacy ConfigMap" "$(named_resource_list configmap "${LEGACY_CONFIGMAPS[@]}")"
    delete_resource_list "Legacy Secret" "$(named_resource_list secret "${LEGACY_SECRETS[@]}")"

    print_progress "4/4" "PVC"
    if [ -n "$DELETE_PVCS" ]; then
        print_warning "已启用 --delete-pvc，将删除旧 Keycloak PVC"
        delete_resource_list "Legacy PVC" "$(named_resource_list pvc "${LEGACY_PVCS[@]}")"
    else
        print_info "未指定 --delete-pvc，保留旧 Keycloak PVC"
    fi
}

run_workloads_only_cleanup() {
    if ! confirm_action "开始清理工作负载？"; then
        print_info "清理已取消"
        exit 0
    fi

    print_header "正在清理工作负载"

    print_progress "1/6" "Jobs"
    delete_jobs

    print_progress "2/6" "Deployments"
    delete_deployments

    print_progress "3/6" "StatefulSets"
    delete_statefulsets

    print_progress "4/6" "HorizontalPodAutoscalers"
    delete_hpas

    print_progress "5/6" "Services"
    delete_services

    print_progress "6/6" "可观测性资源"
    delete_observability_resources

    if [ -n "$RESET_DB" ]; then
        reset_tidb_database "skip-confirm" "附加"
    fi

    if [ -n "$DELETE_SECRETS" ]; then
        interactive_delete_secrets "skip-confirm" "附加"
    fi

    if [ -n "$DELETE_PVCS" ]; then
        interactive_delete_pvcs "skip-confirm" "附加"
    fi

    if [ -n "$DELETE_NAMESPACE" ]; then
        interactive_delete_namespace "skip-confirm" "附加"
    fi
}

main() {
    parse_arguments "$@"

    print_header "Auth9 清理"

    echo -e "${YELLOW}命名空间:${NC} $NAMESPACE"
    echo -e "${YELLOW}模式:${NC} $([ -n "$DRY_RUN" ] && echo "预演模式（不做实际更改）" || echo "交互式")"
    echo ""

    check_namespace
    show_resources
    print_mode_summary

    if [ -n "$DRY_RUN" ]; then
        print_warning "预演模式 - 仅显示将要执行的操作"
    fi

    case "$MODE" in
        legacy-keycloak-only)
            run_legacy_keycloak_cleanup
            ;;
        workloads-only)
            run_workloads_only_cleanup
            ;;
        *)
            run_full_cleanup
            ;;
    esac

    print_header "清理完成"

    if [ -z "$DRY_RUN" ]; then
        echo -e "${YELLOW}'$NAMESPACE' 中的剩余资源:${NC}"
        if kubectl get namespace "$NAMESPACE" &>/dev/null; then
            kubectl get all,secrets,configmaps,pvc -n "$NAMESPACE" 2>/dev/null || echo "  （命名空间已删除）"
        else
            echo "  命名空间已删除"
        fi
    fi

    echo ""
    print_info "重新部署请运行: ./deploy/deploy.sh"
}

main "$@"
