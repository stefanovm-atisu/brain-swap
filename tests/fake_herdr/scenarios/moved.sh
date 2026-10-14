# W-12's pane w4V:p9 is gone; its session now runs in w7Q:p2.
case "$*" in
  "pane get w4V:p9") fail 1 '{"error":{"code":"pane_not_found","message":"pane w4V:p9 not found"},"id":"cli:pane:get"}' ;;
  "workspace list") out '{"id":"cli:workspace:list","result":{"type":"workspace_list","workspaces":[{"active_tab_id":"w4V:t1","workspace_id":"w4V"},{"active_tab_id":"w7Q:t1","workspace_id":"w7Q"}]}}' ;;
  "pane list --workspace w4V") out '{"id":"cli:pane:list","result":{"type":"pane_list","panes":[{"agent_status":"unknown","pane_id":"w4V:p3","tab_id":"w4V:t2","workspace_id":"w4V"},{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"9b1e04d2-5c1f-4b0e-8a57-0d3c2f6e1a90"},"agent_status":"idle","pane_id":"w4V:p11","tab_id":"w4V:t1","workspace_id":"w4V"}]}}' ;;
  "pane list --workspace w7Q") out '{"id":"cli:pane:list","result":{"type":"pane_list","panes":[{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"55583127-a22a-49bc-a803-e777c377a595"},"agent_status":"idle","pane_id":"w7Q:p2","tab_id":"w7Q:t1","workspace_id":"w7Q"}]}}' ;;
  "agent focus w7Q:p2") exit 0 ;;
esac
