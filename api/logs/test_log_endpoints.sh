#!/bin/bash

HOST="http://localhost:8080"

# 1. Listar archivos de logs
echo "1. Listar archivos de logs:"
curl -s "$HOST/logs/files/list" | jq

echo -e "\n2. Obtener las últimas N líneas de todos los logs (ejemplo: 50):"
curl "$HOST/logs/files/tail/50"

echo -e "\n3. Descargar todos los logs como ZIP (zip):"
curl -OJ "$HOST/logs/files/download"

echo -e "\n4. Eliminar todos los archivos de log (el de hoy se trunca):"
curl -X DELETE "$HOST/logs/files/delete"
    
echo -e "\n5. Descargar el archivo de log de hoy (formato: app.log.YYYY-MM-DD):"
TODAY=$(date +app.log.%Y-%m-%d)
curl -OJ "$HOST/logs/files/$TODAY" 

echo -e "\n6. Eliminar el archivo de log de hoy (formato: app.log.YYYY-MM-DD):"
TODAY=$(date +app.log.%Y-%m-%d)
curl -X DELETE "$HOST/logs/files/$TODAY" 