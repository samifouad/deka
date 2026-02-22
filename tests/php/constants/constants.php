<?php
define('PHP_APP', 'php-rs');
const PHP_VERSION = '0.1';

if (!defined('PHP_APP')) {
    echo "missing constant\n";
}

echo PHP_APP . " " . PHP_VERSION . "\n";
