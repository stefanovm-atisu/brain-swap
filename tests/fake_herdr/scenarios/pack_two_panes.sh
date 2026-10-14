# Panes w4V:p9 and w4V:p11 are live and run Claude sessions SESSION_A and SESSION_B
# of tests/common/skill_md.rs.
case "$*" in
  "pane get w4V:p9") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"aaaaaaaa-1111-4111-8111-111111111111"},"agent_status":"idle","pane_id":"w4V:p9","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
  "pane get w4V:p11") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"bbbbbbbb-2222-4222-8222-222222222222"},"agent_status":"idle","pane_id":"w4V:p11","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
esac
