<?php

$input = '{"name":"deka","nums":[1,2,3],"ok":true,"nested":{"a":1}}';
$result = json_decode($input, true);
var_dump($result);

echo json_encode($result), "\n";
var_dump(json_validate($input));

$unicode = '{"check":"\\u2713","snow":"\\u2603"}';
var_dump(json_decode($unicode, true));

$bad = '{bad json}';
var_dump(json_decode($bad, true));
var_dump(json_last_error());
var_dump(json_last_error_msg());

?>
