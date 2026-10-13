# W-12's pane w4V:p9 is live and runs Claude, but every focus fails (SP-5 not run: generic stderr).
case "$*" in
  "pane get w4V:p9") out '{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"agent":"claude","agent_session":{"agent":"claude","kind":"id","source":"herdr:claude","value":"55583127-a22a-49bc-a803-e777c377a595"},"agent_status":"idle","pane_id":"w4V:p9","tab_id":"w4V:t1","workspace_id":"w4V"}}}' ;;
  "agent focus w4V:p9"|"workspace focus w4V") fail 1 'fake focus failure' ;;
esac
