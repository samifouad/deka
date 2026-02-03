<?php
import { returns_result, echo_result } from '@user/bridge_test';

$val = returns_result(5);
var_export($val);
echo "\n";

$val = returns_result(0);
var_export($val);
echo "\n";

echo echo_result(['ok' => true, 'value' => 9]) . "\n";
echo echo_result(['ok' => false, 'error' => 'no']) . "\n";

$ok = new stdClass();
$ok->ok = true;
$ok->value = 11;
echo echo_result($ok) . "\n";

$err = new stdClass();
$err->ok = false;
$err->error = 'bad';
echo echo_result($err) . "\n";
