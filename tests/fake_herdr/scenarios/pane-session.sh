# A Claude pane with agent session herdr-s1.
case "$*" in
  "pane get w4V:p9") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"herdr-s1"},"agent_status":"idle","pane_id":"w4V:p9","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
esac
