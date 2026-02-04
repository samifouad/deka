#include <napi.h>

typedef struct TSLanguage TSLanguage;

extern "C" TSLanguage *tree_sitter_phpx();
extern "C" TSLanguage *tree_sitter_phpx_only();

// "tree-sitter", "language" hashed with BLAKE2
const napi_type_tag LANGUAGE_TYPE_TAG = {
    0x8AF2E5212AD58ABF, 0xD5006CAD83ABBA16
};

Napi::Object Init(Napi::Env env, Napi::Object exports) {
    auto phpx = Napi::Object::New(env);
    phpx["name"] = Napi::String::New(env, "phpx");
    auto phpx_language = Napi::External<TSLanguage>::New(env, tree_sitter_phpx());
    phpx_language.TypeTag(&LANGUAGE_TYPE_TAG);
    phpx["language"] = phpx_language;

    auto phpx_only = Napi::Object::New(env);
    phpx_only["name"] = Napi::String::New(env, "phpx_only");
    auto phpx_only_language = Napi::External<TSLanguage>::New(env, tree_sitter_phpx_only());
    phpx_only_language.TypeTag(&LANGUAGE_TYPE_TAG);
    phpx_only["language"] = phpx_only_language;

    exports["phpx"] = phpx;
    exports["phpx_only"] = phpx_only;
    return exports;
}

NODE_API_MODULE(tree_sitter_phpx_binding, Init)
