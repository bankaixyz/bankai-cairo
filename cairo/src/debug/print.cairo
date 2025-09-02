from starkware.cairo.common.cairo_builtins import UInt384
from starkware.cairo.common.uint256 import Uint256

func info_felt(value: felt) {
    %{ print(f"Info: {ids.value}") %}

    return ();
}

func info_felt_hex(value: felt) {
    %{ print(f"Info: {hex(ids.value)}") %}

    return ();
}

func info_string(value: felt) {
    %{ print(f"Info: {ids.value}") %}

    return ();
}

func info_uint256(value: Uint256) {
    %{ print(f"Info: {hex(ids.value.high * 2**128 + ids.value.low)}") %}

    return ();
}

func info_uint384(value: UInt384) {
    %{ print(f"Info: {hex(ids.value.d3 * 2 ** 144 + ids.value.d2 * 2 ** 96 + ids.value.d1 * 2 ** 48 + ids.value.d0)}") %}

    return ();
}

func info_segment_hex(segment_ptr: felt*, len: felt, index: felt) {
    if (index == len) {
        return ();
    }
    info_string(index);
    info_felt_hex([segment_ptr + index]);
    return info_segment_hex(segment_ptr=segment_ptr, len=len, index=index + 1);
}

func debug_felt(value: felt) {
    %{ print(f"Debug: {ids.value}") %}

    return ();
}

func debug_felt_hex(value: felt) {
    %{ print(f"Debug: {hex(ids.value)}") %}

    return ();
}

func debug_string(value: felt) {
    %{ print(f"Debug: {ids.value}") %}

    return ();
}

func debug_uint256(value: Uint256) {
    %{ print(f"Debug: {hex(ids.value.high * 2**128 + ids.value.low)}") %}

    return ();
}

func debug_uint384(value: UInt384) {
    %{ print(f"Debug: {hex(ids.value.d3 * 2 ** 144 + ids.value.d2 * 2 ** 96 + ids.value.d1 * 2 ** 48 + ids.value.d0)}") %}

    return ();
}

func debug_segment_hex(segment_ptr: felt*, len: felt, index: felt) {
    if (index == len) {
        return ();
    }
    debug_string(index);
    debug_felt_hex([segment_ptr + index]);
    return debug_segment_hex(segment_ptr=segment_ptr, len=len, index=index + 1);
}
