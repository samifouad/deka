<?php
import { takes_object, takes_struct } from '@user/bridge_test';

echo takes_object(['name' => 'Sami', 'age' => 41]) . "\n";
echo takes_object(['name' => 'NoAge']) . "\n";
echo takes_object(['name' => 'Extra', 'age' => 2, 'extra' => 'ok']) . "\n";

$user = new stdClass();
$user->name = 'Kim';
$user->age = 5;
$user->extra = 'ignored';
echo takes_object($user) . "\n";

echo takes_struct(['name' => 'Ada', 'age' => 10]) . "\n";
echo takes_struct(['name' => 'Ada']) . "\n";
echo takes_struct(['name' => 'Extra', 'age' => 11, 'extra' => 1]) . "\n";

$person = new stdClass();
$person->name = 'Bob';
$person->age = 9;
$person->extra = 'x';
echo takes_struct($person) . "\n";
