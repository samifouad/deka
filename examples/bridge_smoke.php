<?php
import { takes_option, takes_object, takes_struct, returns_result, echo_result } from '@user/bridge_test';

$exports = phpx_import('@user/bridge_test');
global $__phpx_types__user_bridge_test;
echo "exports: " . json_encode($exports) . "\n";
echo "types var: " . json_encode($__phpx_types__user_bridge_test) . "\n";
echo "types map: " . json_encode($GLOBALS['__PHPX_TYPES'] ?? null) . "\n";

$results = [];
$results[] = ['takes_option(null)', takes_option(null)];
$results[] = ['takes_option(7)', takes_option(7)];

$results[] = ['takes_object(array)', takes_object(['name' => 'Sami', 'age' => 38, 'extra' => 'ignored'])];
$results[] = ['takes_object(stdClass)', takes_object((object)['name' => 'Ava'])];

$results[] = ['takes_struct(array)', takes_struct(['name' => 'Lia', 'age' => 5, 'extra' => true])];
$results[] = ['takes_struct(stdClass)', takes_struct((object)['name' => 'Milo', 'age' => 2])];

$results[] = ['returns_result(2)', returns_result(2)];
$results[] = ['returns_result(-1)', returns_result(-1)];

$results[] = ['echo_result(ok)', echo_result(['ok' => true, 'value' => 9])];
$results[] = ['echo_result(err)', echo_result(['error' => 'nope'])];

foreach ($results as $row) {
    [$label, $value] = $row;
    if (is_array($value)) {
        $value = json_encode($value);
    }
    echo $label . ' => ' . $value . "\n";
}
