# AS-6: W-12 is parked from w4V:p3, a plain shell pane (SP-4 sample shape) in tab w4V:t2.
case "$*" in
  "pane get w4V:p3") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent_status":"unknown","pane_id":"w4V:p3","tab_id":"w4V:t2","workspace_id":"w4V"}}}' ;;
esac
