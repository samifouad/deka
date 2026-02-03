<?php
import { takes_option, returns_option } from '@user/bridge_test';

echo takes_option(null) . "\n";
echo takes_option(7) . "\n";

$val = returns_option(3);
var_export($val);
echo "\n";

$val = returns_option(0);
var_export($val);
echo "\n";
