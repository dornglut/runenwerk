{{- define "multiplayer-stack.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "multiplayer-stack.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}

{{- define "multiplayer-stack.labels" -}}
app.kubernetes.io/name: {{ include "multiplayer-stack.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
helm.sh/chart: {{ .Chart.Name }}-{{ .Chart.Version | replace "+" "_" }}
{{- end -}}

{{- define "multiplayer-stack.selectorLabels" -}}
app.kubernetes.io/name: {{ include "multiplayer-stack.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end -}}

{{- define "multiplayer-stack.coreApiName" -}}
{{ include "multiplayer-stack.fullname" . }}-core-api
{{- end -}}

{{- define "multiplayer-stack.coreApiServiceName" -}}
{{ include "multiplayer-stack.coreApiName" . }}
{{- end -}}

{{- define "multiplayer-stack.operatorConsoleName" -}}
{{ include "multiplayer-stack.fullname" . }}-operator-console
{{- end -}}

{{- define "multiplayer-stack.operatorConsoleServiceName" -}}
{{ include "multiplayer-stack.operatorConsoleName" . }}
{{- end -}}

{{- define "multiplayer-stack.postgresName" -}}
{{ include "multiplayer-stack.fullname" . }}-postgres
{{- end -}}

{{- define "multiplayer-stack.postgresServiceName" -}}
{{ include "multiplayer-stack.postgresName" . }}
{{- end -}}

{{- define "multiplayer-stack.redisName" -}}
{{ include "multiplayer-stack.fullname" . }}-redis
{{- end -}}

{{- define "multiplayer-stack.redisServiceName" -}}
{{ include "multiplayer-stack.redisName" . }}
{{- end -}}

{{- define "multiplayer-stack.fleetControlName" -}}
{{ include "multiplayer-stack.fullname" . }}-fleet-control
{{- end -}}

{{- define "multiplayer-stack.fleetControlServiceName" -}}
{{ include "multiplayer-stack.fleetControlName" . }}
{{- end -}}

{{- define "multiplayer-stack.grottoServerName" -}}
{{ include "multiplayer-stack.fullname" . }}-grotto-server
{{- end -}}

{{- define "multiplayer-stack.grottoServerServiceName" -}}
{{ include "multiplayer-stack.grottoServerName" . }}
{{- end -}}

{{- define "multiplayer-stack.caddyName" -}}
{{ include "multiplayer-stack.fullname" . }}-caddy
{{- end -}}

{{- define "multiplayer-stack.coreApiEnvSecretName" -}}
{{ include "multiplayer-stack.fullname" . }}-core-api-env
{{- end -}}

{{- define "multiplayer-stack.runtimeBridgeEnvSecretName" -}}
{{ include "multiplayer-stack.fullname" . }}-runtime-bridge-env
{{- end -}}

{{- define "multiplayer-stack.fleetBridgeEnvSecretName" -}}
{{ include "multiplayer-stack.fullname" . }}-fleet-bridge-env
{{- end -}}

{{- define "multiplayer-stack.serverConfigName" -}}
{{ include "multiplayer-stack.fullname" . }}-server-config
{{- end -}}

{{- define "multiplayer-stack.fleetConfigName" -}}
{{ include "multiplayer-stack.fullname" . }}-fleet-config
{{- end -}}
