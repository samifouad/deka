<?php
// Core bridge file. Auto-included by the runtime.

namespace Deka\Internal {
    function __phpx_call(string $moduleId, string $export, array $args = []) {
        if (!function_exists('__phpx_load')) {
            throw new \Exception('phpx bridge unavailable');
        }
        __phpx_load($moduleId);
        if (!isset($GLOBALS['__PHPX_MODULES'][$moduleId])) {
            throw new \Exception('Unknown phpx module: ' . $moduleId);
        }
        $exports = $GLOBALS['__PHPX_MODULES'][$moduleId];
        if (!isset($exports[$export])) {
            throw new \Exception('Unknown phpx export: ' . $moduleId . ':' . $export);
        }
        $fn = $exports[$export];
        $result = $fn(...$args);
        if (function_exists('__phpx_to_php')) {
            return __phpx_to_php($result);
        }
        return $result;
    }

    function __phpx_define_function(string $name, string $moduleId, string $export): void {
        if (function_exists($name)) {
            return;
        }
        $moduleCode = var_export($moduleId, true);
        $exportCode = var_export($export, true);
        $code = "function {$name}(...\$args) { return \\Deka\\Internal\\__phpx_call({$moduleCode}, {$exportCode}, \$args); }";
        eval($code);
    }
}

namespace {
    if (!function_exists('panic')) {
        function panic(string $message): void {
            throw new \Exception($message);
        }
    }

    function option_some($value) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/option');
        }
        return Option::Some($value);
    }

    function option_none() {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/option');
        }
        return Option::None;
    }

    function option_is_some($value): bool {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/option');
        }
        return $value instanceof Option && $value->is_some();
    }

    function option_is_none($value): bool {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/option');
        }
        return $value instanceof Option && $value->is_none();
    }

    function option_unwrap($value) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/option');
        }
        if ($value instanceof Option) {
            return $value->unwrap();
        }
        panic('option_unwrap expects Option');
    }

    function result_ok($value) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        return Result::Ok($value);
    }

    function result_err($error) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        return Result::Err($error);
    }

    function result_is_ok($value): bool {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        return $value instanceof Result && $value->is_ok();
    }

    function result_is_err($value): bool {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        return $value instanceof Result && $value->is_err();
    }

    function result_unwrap($value) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        if ($value instanceof Result) {
            return $value->unwrap();
        }
        panic('result_unwrap expects Result');
    }

    function result_unwrap_or($value, $fallback) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        if ($value instanceof Result) {
            return $value->unwrap_or($fallback);
        }
        return $fallback;
    }

    function try_result(callable $fn) {
        if (function_exists('__phpx_load')) {
            __phpx_load('core/result');
        }
        try {
            return Result::Ok($fn());
        } catch (\Throwable $e) {
            return Result::Err($e);
        }
    }

    // Hand-maintained list of stdlib bindings (function => [moduleId, export]).
    // Keep this list explicit and versioned.
    $GLOBALS['__DEKA_PHPX_STDLIB'] = [
        // json module additions
        'json_decode' => ['json', 'json_decode'],
        'json_decode_result' => ['json', 'json_decode_result'],
        'json_encode' => ['json', 'json_encode'],
        'json_last_error' => ['json', 'json_last_error'],
        'json_last_error_msg' => ['json', 'json_last_error_msg'],
        'json_validate' => ['json', 'json_validate'],

        // array module additions
        'array_all' => ['array/array_all', 'array_all'],
        'array_any' => ['array/array_any', 'array_any'],
        'array_change_key_case' => ['array/array_change_key_case', 'array_change_key_case'],
        'array_chunk' => ['array/array_chunk', 'array_chunk'],
        'array_column' => ['array/array_column', 'array_column'],
        'array_combine' => ['array/array_combine', 'array_combine'],
        'array_count_values' => ['array/array_count_values', 'array_count_values'],
        'array_diff' => ['array/array_diff', 'array_diff'],
        'array_diff_assoc' => ['array/array_diff_assoc', 'array_diff_assoc'],
        'array_diff_key' => ['array/array_diff_key', 'array_diff_key'],
        'array_diff_uassoc' => ['array/array_diff_uassoc', 'array_diff_uassoc'],
        'array_diff_ukey' => ['array/array_diff_ukey', 'array_diff_ukey'],
        'array_fill' => ['array/array_fill', 'array_fill'],
        'array_fill_keys' => ['array/array_fill_keys', 'array_fill_keys'],
        'array_filter' => ['array/array_filter', 'array_filter'],
        'array_find' => ['array/array_find', 'array_find'],
        'array_find_key' => ['array/array_find_key', 'array_find_key'],
        'array_first' => ['array/array_first', 'array_first'],
        'array_flip' => ['array/array_flip', 'array_flip'],
        'array_intersect' => ['array/array_intersect', 'array_intersect'],
        'array_intersect_assoc' => ['array/array_intersect_assoc', 'array_intersect_assoc'],
        'array_intersect_key' => ['array/array_intersect_key', 'array_intersect_key'],
        'array_intersect_uassoc' => ['array/array_intersect_uassoc', 'array_intersect_uassoc'],
        'array_intersect_ukey' => ['array/array_intersect_ukey', 'array_intersect_ukey'],
        'array_is_list' => ['array/array_is_list', 'array_is_list'],
        'array_key_exists' => ['array/array_key_exists', 'array_key_exists'],
        'array_key_first' => ['array/array_key_first', 'array_key_first'],
        'array_key_last' => ['array/array_key_last', 'array_key_last'],
        'array_keys' => ['array/array_keys', 'array_keys'],
        'array_last' => ['array/array_last', 'array_last'],
        'array_map' => ['array/array_map', 'array_map'],
        'array_merge' => ['array/array_merge', 'array_merge'],
        'array_merge_recursive' => ['array/array_merge_recursive', 'array_merge_recursive'],
        'array_multisort' => ['array/array_multisort', 'array_multisort'],
        'array_pad' => ['array/array_pad', 'array_pad'],
        'array_pop' => ['array/array_pop', 'array_pop'],
        'array_product' => ['array/array_product', 'array_product'],
        'array_push' => ['array/array_push', 'array_push'],
        'array_rand' => ['array/array_rand', 'array_rand'],
        'array_reduce' => ['array/array_reduce', 'array_reduce'],
        'array_replace' => ['array/array_replace', 'array_replace'],
        'array_replace_recursive' => ['array/array_replace_recursive', 'array_replace_recursive'],
        'array_reverse' => ['array/array_reverse', 'array_reverse'],
        'array_search' => ['array/array_search', 'array_search'],
        'array_shift' => ['array/array_shift', 'array_shift'],
        'array_slice' => ['array/array_slice', 'array_slice'],
        'array_splice' => ['array/array_splice', 'array_splice'],
        'array_sum' => ['array/array_sum', 'array_sum'],
        'array_udiff' => ['array/array_udiff', 'array_udiff'],
        'array_udiff_assoc' => ['array/array_udiff_assoc', 'array_udiff_assoc'],
        'array_udiff_uassoc' => ['array/array_udiff_uassoc', 'array_udiff_uassoc'],
        'array_uintersect' => ['array/array_uintersect', 'array_uintersect'],
        'array_uintersect_assoc' => ['array/array_uintersect_assoc', 'array_uintersect_assoc'],
        'array_uintersect_uassoc' => ['array/array_uintersect_uassoc', 'array_uintersect_uassoc'],
        'array_unique' => ['array/array_unique', 'array_unique'],
        'array_unshift' => ['array/array_unshift', 'array_unshift'],
        'array_values' => ['array/array_values', 'array_values'],
        'array_walk' => ['array/array_walk', 'array_walk'],
        'array_walk_recursive' => ['array/array_walk_recursive', 'array_walk_recursive'],
        'arsort' => ['array/arsort', 'arsort'],
        'asort' => ['array/asort', 'asort'],
        'compact' => ['array/compact', 'compact'],
        'count' => ['array/count', 'count'],
        'current' => ['array/current', 'current'],
        'end' => ['array/end', 'end'],
        'extract' => ['array/extract', 'extract'],
        'in_array' => ['array/in_array', 'in_array'],
        'key' => ['array/key', 'key'],
        'key_exists' => ['array/key_exists', 'key_exists'],
        'krsort' => ['array/krsort', 'krsort'],
        'ksort' => ['array/ksort', 'ksort'],
        'natcasesort' => ['array/natcasesort', 'natcasesort'],
        'natsort' => ['array/natsort', 'natsort'],
        'next' => ['array/next', 'next'],
        'pos' => ['array/pos', 'pos'],
        'prev' => ['array/prev', 'prev'],
        'range' => ['array/range', 'range'],
        'reset' => ['array/reset', 'reset'],
        'rsort' => ['array/rsort', 'rsort'],
        'shuffle' => ['array/shuffle', 'shuffle'],
        'sizeof' => ['array/sizeof', 'sizeof'],
        'sort' => ['array/sort', 'sort'],
        'uasort' => ['array/uasort', 'uasort'],
        'uksort' => ['array/uksort', 'uksort'],
        'usort' => ['array/usort', 'usort'],

        // string module additions
        'addcslashes' => ['string/addcslashes', 'addcslashes'],
        'addslashes' => ['string/addslashes', 'addslashes'],
        'bin2hex' => ['string/bin2hex', 'bin2hex'],
        'chop' => ['string/chop', 'chop'],
        'chr' => ['string/chr', 'chr'],
        'chunk_split' => ['string/chunk_split', 'chunk_split'],
        'convert_cyr_string' => ['string/convert_cyr_string', 'convert_cyr_string'],
        'convert_uudecode' => ['string/convert_uudecode', 'convert_uudecode'],
        'convert_uuencode' => ['string/convert_uuencode', 'convert_uuencode'],
        'count_chars' => ['string/count_chars', 'count_chars'],
        'crc32' => ['string/crc32', 'crc32'],
        'crypt' => ['string/crypt', 'crypt'],
        'explode' => ['string/explode', 'explode'],
        'fprintf' => ['string/fprintf', 'fprintf'],
        'hebrev' => ['string/hebrev', 'hebrev'],
        'hex2bin' => ['string/hex2bin', 'hex2bin'],
        'html_entity_decode' => ['string/html_entity_decode', 'html_entity_decode'],
        'htmlentities' => ['string/htmlentities', 'htmlentities'],
        'htmlspecialchars' => ['string/htmlspecialchars', 'htmlspecialchars'],
        'htmlspecialchars_decode' => ['string/htmlspecialchars_decode', 'htmlspecialchars_decode'],
        'implode' => ['string/implode', 'implode'],
        'lcfirst' => ['string/lcfirst', 'lcfirst'],
        'levenshtein' => ['string/levenshtein', 'levenshtein'],
        'localeconv' => ['string/localeconv', 'localeconv'],
        'ltrim' => ['string/ltrim', 'ltrim'],
        'md5' => ['string/md5', 'md5'],
        'md5_file' => ['string/md5_file', 'md5_file'],
        'metaphone' => ['string/metaphone', 'metaphone'],
        'money_format' => ['string/money_format', 'money_format'],
        'nl2br' => ['string/nl2br', 'nl2br'],
        'nl_langinfo' => ['string/nl_langinfo', 'nl_langinfo'],
        'number_format' => ['string/number_format', 'number_format'],
        'ord' => ['string/ord', 'ord'],
        'parse_str' => ['string/parse_str', 'parse_str'],
        'printf' => ['string/printf', 'printf'],
        'quotemeta' => ['string/quotemeta', 'quotemeta'],
        'rtrim' => ['string/rtrim', 'rtrim'],
        'setlocale' => ['string/setlocale', 'setlocale'],
        'sha1' => ['string/sha1', 'sha1'],
        'sha1_file' => ['string/sha1_file', 'sha1_file'],
        'similar_text' => ['string/similar_text', 'similar_text'],
        'soundex' => ['string/soundex', 'soundex'],
        'sprintf' => ['string/sprintf', 'sprintf'],
        'sscanf' => ['string/sscanf', 'sscanf'],
        'str_contains' => ['string/str_contains', 'str_contains'],
        'str_decrement' => ['string/str_decrement', 'str_decrement'],
        'str_ends_with' => ['string/str_ends_with', 'str_ends_with'],
        'str_getcsv' => ['string/str_getcsv', 'str_getcsv'],
        'str_increment' => ['string/str_increment', 'str_increment'],
        'str_ireplace' => ['string/str_ireplace', 'str_ireplace'],
        'str_pad' => ['string/str_pad', 'str_pad'],
        'str_repeat' => ['string/str_repeat', 'str_repeat'],
        'str_replace' => ['string/str_replace', 'str_replace'],
        'str_rot13' => ['string/str_rot13', 'str_rot13'],
        'str_shuffle' => ['string/str_shuffle', 'str_shuffle'],
        'str_split' => ['string/str_split', 'str_split'],
        'str_starts_with' => ['string/str_starts_with', 'str_starts_with'],
        'str_word_count' => ['string/str_word_count', 'str_word_count'],
        'strcasecmp' => ['string/strcasecmp', 'strcasecmp'],
        'strchr' => ['string/strchr', 'strchr'],
        'strcmp' => ['string/strcmp', 'strcmp'],
        'strcoll' => ['string/strcoll', 'strcoll'],
        'strcspn' => ['string/strcspn', 'strcspn'],
        'strip_tags' => ['string/strip_tags', 'strip_tags'],
        'stripcslashes' => ['string/stripcslashes', 'stripcslashes'],
        'stripos' => ['string/stripos', 'stripos'],
        'stripslashes' => ['string/stripslashes', 'stripslashes'],
        'stristr' => ['string/stristr', 'stristr'],
        'strlen' => ['string/strlen', 'strlen'],
        'strnatcasecmp' => ['string/strnatcasecmp', 'strnatcasecmp'],
        'strnatcmp' => ['string/strnatcmp', 'strnatcmp'],
        'strncasecmp' => ['string/strncasecmp', 'strncasecmp'],
        'strncmp' => ['string/strncmp', 'strncmp'],
        'strpbrk' => ['string/strpbrk', 'strpbrk'],
        'strpos' => ['string/strpos', 'strpos'],
        'strrchr' => ['string/strrchr', 'strrchr'],
        'strrev' => ['string/strrev', 'strrev'],
        'strripos' => ['string/strripos', 'strripos'],
        'strrpos' => ['string/strrpos', 'strrpos'],
        'strspn' => ['string/strspn', 'strspn'],
        'strstr' => ['string/strstr', 'strstr'],
        'strtok' => ['string/strtok', 'strtok'],
        'strtolower' => ['string/strtolower', 'strtolower'],
        'strtoupper' => ['string/strtoupper', 'strtoupper'],
        'strtr' => ['string/strtr', 'strtr'],
        'substr' => ['string/substr', 'substr'],
        'substr_compare' => ['string/substr_compare', 'substr_compare'],
        'substr_count' => ['string/substr_count', 'substr_count'],
        'substr_replace' => ['string/substr_replace', 'substr_replace'],
        'trim' => ['string/trim', 'trim'],
        'ucfirst' => ['string/ucfirst', 'ucfirst'],
        'ucwords' => ['string/ucwords', 'ucwords'],
        'utf8_decode' => ['string/utf8_decode', 'utf8_decode'],
        'utf8_encode' => ['string/utf8_encode', 'utf8_encode'],
        'version_compare' => ['string/version_compare', 'version_compare'],
        'vfprintf' => ['string/vfprintf', 'vfprintf'],
        'vprintf' => ['string/vprintf', 'vprintf'],
        'vsprintf' => ['string/vsprintf', 'vsprintf'],
        'wordwrap' => ['string/wordwrap', 'wordwrap'],
    ];

    if (!defined('__DEKA_PHPX_ENTRY')) {
        $lazyModules = isset($GLOBALS['__PHPX_LAZY']) && is_array($GLOBALS['__PHPX_LAZY'])
            ? $GLOBALS['__PHPX_LAZY']
            : null;
        foreach ($GLOBALS['__DEKA_PHPX_STDLIB'] as $name => $binding) {
            $moduleId = $binding[0];
            if ($lazyModules !== null && !isset($lazyModules[$moduleId])) {
                continue;
            }
            $export = $binding[1];
            \Deka\Internal\__phpx_define_function($name, $moduleId, $export);
        }
    }
}
