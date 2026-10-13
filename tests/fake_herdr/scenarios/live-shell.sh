# W-12's pane w4V:p9 is live and runs a plain shell (SP-4 sample shape).
case "$*" in
  "pane get w4V:p9") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent_status":"unknown","pane_id":"w4V:p9","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
  "workspace focus w4V") exit 0 ;;
  "tab focus w4V:t1") exit 0 ;;
esac
