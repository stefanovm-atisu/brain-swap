# W-12's pane w4V:p9 is live and runs a plain shell; so does W-14's
# oldest pane w4V:p3 in tab w4V:t2.
case "$*" in
  "pane get w4V:p9") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent_status":"unknown","pane_id":"w4V:p9","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
  "pane get w4V:p3") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent_status":"unknown","pane_id":"w4V:p3","tab_id":"w4V:t2","workspace_id":"w4V"}}}' ;;
  "workspace focus w4V") exit 0 ;;
  "tab focus w4V:t1") exit 0 ;;
  "tab focus w4V:t2") exit 0 ;;
esac
