#[cfg(test)]
mod tests_assignability_matrix;
#[cfg(test)]
mod tests_assignment_lists;
#[cfg(test)]
mod tests_benchmark_suite;
#[cfg(test)]
mod tests_browser_acceptance_corpus;
#[cfg(test)]
mod tests_builder_contexts;
#[cfg(test)]
mod tests_builtins;
#[cfg(test)]
mod tests_bytecode_vm_spec;
#[cfg(test)]
mod tests_channels;
#[cfg(test)]
mod tests_ci_matrix;
#[cfg(test)]
mod tests_closure_typing;
#[cfg(test)]
mod tests_collections;
#[cfg(test)]
mod tests_collections_more;
#[cfg(test)]
mod tests_comparison_semantics;
#[cfg(test)]
mod tests_compiler_phases;
#[cfg(test)]
mod tests_concurrency;
#[cfg(test)]
mod tests_const;
#[cfg(test)]
mod tests_control_flow;
#[cfg(test)]
mod tests_conversions;
#[cfg(test)]
mod tests_defer;
#[cfg(test)]
mod tests_diagnostics;
#[cfg(test)]
mod tests_diagnostics_messages;
#[cfg(test)]
mod tests_differential_release_gate;
#[cfg(test)]
mod tests_embedding_parity;
#[cfg(test)]
mod tests_flaky_repeat_runner;
#[cfg(test)]
mod tests_function_signature_structural;
#[cfg(test)]
mod tests_functions;
#[cfg(test)]
mod tests_gc_stress;
#[cfg(test)]
mod tests_generic_constraints;
#[cfg(test)]
mod tests_generic_instantiation_identity;
#[cfg(test)]
mod tests_generics_builtins_stdlib;
#[cfg(test)]
mod tests_generics_regressions;
#[cfg(test)]
mod tests_generics_reuse;
#[cfg(test)]
mod tests_generics_type_ids;
#[cfg(test)]
mod tests_globals;
#[cfg(test)]
mod tests_go;
#[cfg(test)]
mod tests_goroutine_patterns;
#[cfg(test)]
mod tests_goroutine_stdlib;
#[cfg(test)]
mod tests_import_resolution;
#[cfg(test)]
mod tests_imported_package_release_gate;
#[cfg(test)]
mod tests_imported_packages;
#[cfg(test)]
mod tests_interface_method_sets;
#[cfg(test)]
mod tests_interfaces;
#[cfg(test)]
mod tests_interfaces_more;
#[cfg(test)]
mod tests_interfaces_parity;
#[cfg(test)]
mod tests_map_key_comparability;
#[cfg(test)]
mod tests_maps;
#[cfg(test)]
mod tests_method_values;
#[cfg(test)]
mod tests_multi_result_assignment_types;
#[cfg(test)]
mod tests_multi_results;
#[cfg(test)]
mod tests_nil_matrix;
#[cfg(test)]
mod tests_negative_support_corpus;
#[cfg(test)]
mod tests_operators;
#[cfg(test)]
mod tests_package_artifact_schema;
#[cfg(test)]
mod tests_package_init_order;
#[cfg(test)]
mod tests_parity_corpus;
#[cfg(test)]
mod tests_parity_release_gate;
#[cfg(test)]
mod tests_pointers;
#[cfg(test)]
mod tests_range;
#[cfg(test)]
mod tests_rebuild_graph;
#[cfg(test)]
mod tests_reflect_json_regressions;
#[cfg(test)]
mod tests_release_artifact_reproducibility;
#[cfg(test)]
mod tests_release_gate;
#[cfg(test)]
mod tests_replay;
#[cfg(test)]
mod tests_runtime_errors;
#[cfg(test)]
mod tests_runtime_language_backlog;
#[cfg(test)]
mod tests_runtime_type_inventory;
#[cfg(test)]
mod tests_select;
#[cfg(test)]
mod tests_select_close_paths;
#[cfg(test)]
mod tests_select_more;
#[cfg(test)]
mod tests_select_nil_cases;
#[cfg(test)]
mod tests_selectors;
#[cfg(test)]
mod tests_semantic_differential;
#[cfg(test)]
mod tests_slice_array_semantics;
#[cfg(test)]
mod tests_stdlib;
#[cfg(test)]
mod tests_stdlib_base64;
#[cfg(test)]
mod tests_stdlib_bytes;
#[cfg(test)]
mod tests_stdlib_bytes_func;
#[cfg(test)]
mod tests_stdlib_cmp;
#[cfg(test)]
mod tests_stdlib_collection_differential;
#[cfg(test)]
mod tests_stdlib_collection_generics;
#[cfg(test)]
mod tests_stdlib_context;
#[cfg(test)]
mod tests_stdlib_crypto_encoding_differential;
#[cfg(test)]
mod tests_stdlib_differential_support;
#[cfg(test)]
mod tests_stdlib_errors;
#[cfg(test)]
mod tests_stdlib_errors_differential;
#[cfg(test)]
mod tests_stdlib_filepath;
#[cfg(test)]
mod tests_stdlib_fmt;
#[cfg(test)]
mod tests_stdlib_fmt_print;
#[cfg(test)]
mod tests_stdlib_function_values;
#[cfg(test)]
mod tests_stdlib_hex;
#[cfg(test)]
mod tests_stdlib_io_fs;
#[cfg(test)]
mod tests_stdlib_io_fs_differential;
#[cfg(test)]
mod tests_stdlib_io_fs_metadata;
#[cfg(test)]
mod tests_stdlib_json;
#[cfg(test)]
mod tests_stdlib_json_differential;
#[cfg(test)]
mod tests_stdlib_json_interfaces;
#[cfg(test)]
mod tests_stdlib_json_wrappers;
#[cfg(test)]
mod tests_stdlib_log;
#[cfg(test)]
mod tests_stdlib_log_differential;
#[cfg(test)]
mod tests_stdlib_maps;
#[cfg(test)]
mod tests_stdlib_math;
#[cfg(test)]
mod tests_stdlib_math_bits;
#[cfg(test)]
mod tests_stdlib_math_differential;
#[cfg(test)]
mod tests_stdlib_math_more;
#[cfg(test)]
mod tests_stdlib_md5;
#[cfg(test)]
mod tests_stdlib_min_max_clear;
#[cfg(test)]
mod tests_stdlib_native_go_differential;
#[cfg(test)]
mod tests_stdlib_net_http;
#[cfg(test)]
mod tests_stdlib_net_http_clone_reuse;
#[cfg(test)]
mod tests_stdlib_net_http_fidelity;
#[cfg(test)]
mod tests_stdlib_net_http_more;
#[cfg(test)]
mod tests_stdlib_net_http_support;
#[cfg(test)]
mod tests_stdlib_net_url;
#[cfg(test)]
mod tests_stdlib_net_url_differential;
#[cfg(test)]
mod tests_stdlib_net_url_userinfo;
#[cfg(test)]
mod tests_stdlib_os;
#[cfg(test)]
mod tests_stdlib_os_error_contracts;
#[cfg(test)]
mod tests_stdlib_os_mutation;
#[cfg(test)]
mod tests_stdlib_os_paths;
#[cfg(test)]
mod tests_stdlib_parse_uint;
#[cfg(test)]
mod tests_stdlib_path;
#[cfg(test)]
mod tests_stdlib_rand;
#[cfg(test)]
mod tests_stdlib_reflect;
#[cfg(test)]
mod tests_stdlib_reflect_differential;
#[cfg(test)]
mod tests_stdlib_regexp;
#[cfg(test)]
mod tests_stdlib_regexp_differential;
#[cfg(test)]
mod tests_stdlib_sha1;
#[cfg(test)]
mod tests_stdlib_sha256;
#[cfg(test)]
mod tests_stdlib_sha512;
#[cfg(test)]
mod tests_stdlib_slices;
#[cfg(test)]
mod tests_stdlib_sort;
#[cfg(test)]
mod tests_stdlib_sort_float64s;
#[cfg(test)]
mod tests_stdlib_sort_search;
#[cfg(test)]
mod tests_stdlib_sort_slice;
#[cfg(test)]
mod tests_stdlib_strconv;
#[cfg(test)]
mod tests_stdlib_strconv_differential;
#[cfg(test)]
mod tests_stdlib_strings_fold;
#[cfg(test)]
mod tests_stdlib_strings_func;
#[cfg(test)]
mod tests_stdlib_strings_map;
#[cfg(test)]
mod tests_stdlib_strings_replacer;
#[cfg(test)]
mod tests_stdlib_strings_split_after;
#[cfg(test)]
mod tests_stdlib_strings_split_after_n;
#[cfg(test)]
mod tests_stdlib_strings_splitn;
#[cfg(test)]
mod tests_stdlib_strings_to_title;
#[cfg(test)]
mod tests_stdlib_sync;
#[cfg(test)]
mod tests_stdlib_text_differential;
#[cfg(test)]
mod tests_stdlib_time;
#[cfg(test)]
mod tests_stdlib_time_context_http_differential;
#[cfg(test)]
mod tests_stdlib_unicode;

#[cfg(test)]
mod tests_switch;
#[cfg(test)]
mod tests_type_assert_more;
#[cfg(test)]
mod tests_type_assert_switch_typing;
#[cfg(test)]
mod tests_type_system_design;
#[cfg(test)]
mod tests_types;
#[cfg(test)]
mod tests_unwind;
#[cfg(test)]
mod tests_unwind_matrix;
#[cfg(test)]
mod tests_var;
#[cfg(test)]
mod tests_var_more;
#[cfg(test)]
mod tests_wasm_worker_protocol_spec;
