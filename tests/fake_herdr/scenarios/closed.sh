# W-12's pane w4V:p9 is gone and no listed pane runs its session.
case "$*" in
  "pane get w4V:p9") fail 1 '{"error":{"code":"pane_not_found","message":"pane w4V:p9 not found"},"id":"cli:pane:get"}' ;;
  "workspace list") out '{"id":"cli:workspace:list","result":{"type":"workspace_list","workspaces":[{"active_tab_id":"w4V:t1","workspace_id":"w4V"}]}}' ;;
  "pane list --workspace w4V") out '{"id":"cli:pane:list","result":{"type":"pane_list","panes":[{"agent_status":"unknown","pane_id":"w4V:p3","tab_id":"w4V:t2","workspace_id":"w4V"},{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"9b1e04d2-5c1f-4b0e-8a57-0d3c2f6e1a90"},"agent_status":"idle","pane_id":"w4V:p11","tab_id":"w4V:t1","workspace_id":"w4V"}]}}' ;;
esac
