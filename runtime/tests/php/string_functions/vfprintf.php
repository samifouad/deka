<?php
$fp = fopen('php://memory', 'r+');
vfprintf($fp, "%s", ['php']);
rewind($fp);
echo stream_get_contents($fp);
