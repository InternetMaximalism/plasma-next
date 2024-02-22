// SPDX-License-Identifier: MIT
// solhint-disable no-unused-vars
// solhint-disable no-inline-assembly
pragma solidity 0.8.23;

contract Halo2Verifier {
    uint256 internal constant PROOF_LEN_CPTR = 0x64;
    uint256 internal constant PROOF_CPTR = 0x84;
    uint256 internal constant NUM_INSTANCE_CPTR = 0x1c04;
    uint256 internal constant INSTANCE_CPTR = 0x1c24;

    uint256 internal constant FIRST_QUOTIENT_X_CPTR = 0x0d04;
    uint256 internal constant LAST_QUOTIENT_X_CPTR = 0x0e04;

    uint256 internal constant VK_MPTR = 0x05a0;
    uint256 internal constant VK_DIGEST_MPTR = 0x05a0;
    uint256 internal constant NUM_INSTANCES_MPTR = 0x05c0;
    uint256 internal constant K_MPTR = 0x05e0;
    uint256 internal constant N_INV_MPTR = 0x0600;
    uint256 internal constant OMEGA_MPTR = 0x0620;
    uint256 internal constant OMEGA_INV_MPTR = 0x0640;
    uint256 internal constant OMEGA_INV_TO_L_MPTR = 0x0660;
    uint256 internal constant HAS_ACCUMULATOR_MPTR = 0x0680;
    uint256 internal constant ACC_OFFSET_MPTR = 0x06a0;
    uint256 internal constant NUM_ACC_LIMBS_MPTR = 0x06c0;
    uint256 internal constant NUM_ACC_LIMB_BITS_MPTR = 0x06e0;
    uint256 internal constant G1_X_MPTR = 0x0700;
    uint256 internal constant G1_Y_MPTR = 0x0720;
    uint256 internal constant G2_X_1_MPTR = 0x0740;
    uint256 internal constant G2_X_2_MPTR = 0x0760;
    uint256 internal constant G2_Y_1_MPTR = 0x0780;
    uint256 internal constant G2_Y_2_MPTR = 0x07a0;
    uint256 internal constant NEG_S_G2_X_1_MPTR = 0x07c0;
    uint256 internal constant NEG_S_G2_X_2_MPTR = 0x07e0;
    uint256 internal constant NEG_S_G2_Y_1_MPTR = 0x0800;
    uint256 internal constant NEG_S_G2_Y_2_MPTR = 0x0820;

    uint256 internal constant CHALLENGE_MPTR = 0x0e00;

    uint256 internal constant THETA_MPTR = 0x0e00;
    uint256 internal constant BETA_MPTR = 0x0e20;
    uint256 internal constant GAMMA_MPTR = 0x0e40;
    uint256 internal constant Y_MPTR = 0x0e60;
    uint256 internal constant X_MPTR = 0x0e80;
    uint256 internal constant ZETA_MPTR = 0x0ea0;
    uint256 internal constant NU_MPTR = 0x0ec0;
    uint256 internal constant MU_MPTR = 0x0ee0;

    uint256 internal constant ACC_LHS_X_MPTR = 0x0f00;
    uint256 internal constant ACC_LHS_Y_MPTR = 0x0f20;
    uint256 internal constant ACC_RHS_X_MPTR = 0x0f40;
    uint256 internal constant ACC_RHS_Y_MPTR = 0x0f60;
    uint256 internal constant X_N_MPTR = 0x0f80;
    uint256 internal constant X_N_MINUS_1_INV_MPTR = 0x0fa0;
    uint256 internal constant L_LAST_MPTR = 0x0fc0;
    uint256 internal constant L_BLIND_MPTR = 0x0fe0;
    uint256 internal constant L_0_MPTR = 0x1000;
    uint256 internal constant INSTANCE_EVAL_MPTR = 0x1020;
    uint256 internal constant QUOTIENT_EVAL_MPTR = 0x1040;
    uint256 internal constant QUOTIENT_X_MPTR = 0x1060;
    uint256 internal constant QUOTIENT_Y_MPTR = 0x1080;
    uint256 internal constant R_EVAL_MPTR = 0x10a0;
    uint256 internal constant PAIRING_LHS_X_MPTR = 0x10c0;
    uint256 internal constant PAIRING_LHS_Y_MPTR = 0x10e0;
    uint256 internal constant PAIRING_RHS_X_MPTR = 0x1100;
    uint256 internal constant PAIRING_RHS_Y_MPTR = 0x1120;

    function verifyProof(
        address vk,
        bytes calldata proof,
        uint256[] calldata instances
    ) public view returns (bool) {
        assembly {
            // Read EC point (x, y) at (proof_cptr, proof_cptr + 0x20),
            // and check if the point is on affine plane,
            // and store them in (hash_mptr, hash_mptr + 0x20).
            // Return updated (success, proof_cptr, hash_mptr).
            function read_ec_point(success, proof_cptr, hash_mptr, q)
                -> ret0, ret1, ret2
            {
                let x := calldataload(proof_cptr)
                let y := calldataload(add(proof_cptr, 0x20))
                ret0 := and(success, lt(x, q))
                ret0 := and(ret0, lt(y, q))
                ret0 := and(
                    ret0,
                    eq(
                        mulmod(y, y, q),
                        addmod(mulmod(x, mulmod(x, x, q), q), 3, q)
                    )
                )
                mstore(hash_mptr, x)
                mstore(add(hash_mptr, 0x20), y)
                ret1 := add(proof_cptr, 0x40)
                ret2 := add(hash_mptr, 0x40)
            }

            // Squeeze challenge by keccak256(memory[0..hash_mptr]),
            // and store hash mod r as challenge in challenge_mptr,
            // and push back hash in 0x00 as the first input for next squeeze.
            // Return updated (challenge_mptr, hash_mptr).
            function squeeze_challenge(challenge_mptr, hash_mptr, r)
                -> ret0, ret1
            {
                let hash := keccak256(0x00, hash_mptr)
                mstore(challenge_mptr, mod(hash, r))
                mstore(0x00, hash)
                ret0 := add(challenge_mptr, 0x20)
                ret1 := 0x20
            }

            // Squeeze challenge without absorbing new input from calldata,
            // by putting an extra 0x01 in memory[0x20] and squeeze by keccak256(memory[0..21]),
            // and store hash mod r as challenge in challenge_mptr,
            // and push back hash in 0x00 as the first input for next squeeze.
            // Return updated (challenge_mptr).
            function squeeze_challenge_cont(challenge_mptr, r) -> ret {
                mstore8(0x20, 0x01)
                let hash := keccak256(0x00, 0x21)
                mstore(challenge_mptr, mod(hash, r))
                mstore(0x00, hash)
                ret := add(challenge_mptr, 0x20)
            }

            // Batch invert values in memory[mptr_start..mptr_end] in place.
            // Return updated (success).
            function batch_invert(success, mptr_start, mptr_end, r) -> ret {
                let gp_mptr := mptr_end
                let gp := mload(mptr_start)
                let mptr := add(mptr_start, 0x20)
                for {

                } lt(mptr, sub(mptr_end, 0x20)) {

                } {
                    gp := mulmod(gp, mload(mptr), r)
                    mstore(gp_mptr, gp)
                    mptr := add(mptr, 0x20)
                    gp_mptr := add(gp_mptr, 0x20)
                }
                gp := mulmod(gp, mload(mptr), r)

                mstore(gp_mptr, 0x20)
                mstore(add(gp_mptr, 0x20), 0x20)
                mstore(add(gp_mptr, 0x40), 0x20)
                mstore(add(gp_mptr, 0x60), gp)
                mstore(add(gp_mptr, 0x80), sub(r, 2))
                mstore(add(gp_mptr, 0xa0), r)
                ret := and(
                    success,
                    staticcall(gas(), 0x05, gp_mptr, 0xc0, gp_mptr, 0x20)
                )
                let all_inv := mload(gp_mptr)

                let first_mptr := mptr_start
                let second_mptr := add(first_mptr, 0x20)
                gp_mptr := sub(gp_mptr, 0x20)
                for {

                } lt(second_mptr, mptr) {

                } {
                    let inv := mulmod(all_inv, mload(gp_mptr), r)
                    all_inv := mulmod(all_inv, mload(mptr), r)
                    mstore(mptr, inv)
                    mptr := sub(mptr, 0x20)
                    gp_mptr := sub(gp_mptr, 0x20)
                }
                let inv_first := mulmod(all_inv, mload(second_mptr), r)
                let inv_second := mulmod(all_inv, mload(first_mptr), r)
                mstore(first_mptr, inv_first)
                mstore(second_mptr, inv_second)
            }

            // Add (x, y) into point at (0x00, 0x20).
            // Return updated (success).
            function ec_add_acc(success, x, y) -> ret {
                mstore(0x40, x)
                mstore(0x60, y)
                ret := and(
                    success,
                    staticcall(gas(), 0x06, 0x00, 0x80, 0x00, 0x40)
                )
            }

            // Scale point at (0x00, 0x20) by scalar.
            function ec_mul_acc(success, scalar) -> ret {
                mstore(0x40, scalar)
                ret := and(
                    success,
                    staticcall(gas(), 0x07, 0x00, 0x60, 0x00, 0x40)
                )
            }

            // Add (x, y) into point at (0x80, 0xa0).
            // Return updated (success).
            function ec_add_tmp(success, x, y) -> ret {
                mstore(0xc0, x)
                mstore(0xe0, y)
                ret := and(
                    success,
                    staticcall(gas(), 0x06, 0x80, 0x80, 0x80, 0x40)
                )
            }

            // Scale point at (0x80, 0xa0) by scalar.
            // Return updated (success).
            function ec_mul_tmp(success, scalar) -> ret {
                mstore(0xc0, scalar)
                ret := and(
                    success,
                    staticcall(gas(), 0x07, 0x80, 0x60, 0x80, 0x40)
                )
            }

            // Perform pairing check.
            // Return updated (success).
            function ec_pairing(success, lhs_x, lhs_y, rhs_x, rhs_y) -> ret {
                mstore(0x00, lhs_x)
                mstore(0x20, lhs_y)
                mstore(0x40, mload(G2_X_1_MPTR))
                mstore(0x60, mload(G2_X_2_MPTR))
                mstore(0x80, mload(G2_Y_1_MPTR))
                mstore(0xa0, mload(G2_Y_2_MPTR))
                mstore(0xc0, rhs_x)
                mstore(0xe0, rhs_y)
                mstore(0x100, mload(NEG_S_G2_X_1_MPTR))
                mstore(0x120, mload(NEG_S_G2_X_2_MPTR))
                mstore(0x140, mload(NEG_S_G2_Y_1_MPTR))
                mstore(0x160, mload(NEG_S_G2_Y_2_MPTR))
                ret := and(
                    success,
                    staticcall(gas(), 0x08, 0x00, 0x180, 0x00, 0x20)
                )
                ret := and(ret, mload(0x00))
            }

            // Modulus
            let
                q
            := 21888242871839275222246405745257275088696311157297823662689037894645226208583 // BN254 base field
            let
                r
            := 21888242871839275222246405745257275088548364400416034343698204186575808495617 // BN254 scalar field

            // Initialize success as true
            let success := true

            {
                // Copy vk_digest and num_instances of vk into memory
                extcodecopy(vk, VK_MPTR, 0x00, 0x40)

                // Check valid length of proof
                success := and(
                    success,
                    eq(0x1b80, calldataload(PROOF_LEN_CPTR))
                )

                // Check valid length of instances
                let num_instances := mload(NUM_INSTANCES_MPTR)
                success := and(
                    success,
                    eq(num_instances, calldataload(NUM_INSTANCE_CPTR))
                )

                // Absorb vk diegst
                mstore(0x00, mload(VK_DIGEST_MPTR))

                // Read instances and witness commitments and generate challenges
                let hash_mptr := 0x20
                let instance_cptr := INSTANCE_CPTR
                for {
                    let instance_cptr_end := add(
                        instance_cptr,
                        mul(0x20, num_instances)
                    )
                } lt(instance_cptr, instance_cptr_end) {

                } {
                    let instance := calldataload(instance_cptr)
                    success := and(success, lt(instance, r))
                    mstore(hash_mptr, instance)
                    instance_cptr := add(instance_cptr, 0x20)
                    hash_mptr := add(hash_mptr, 0x20)
                }

                let proof_cptr := PROOF_CPTR
                let challenge_mptr := CHALLENGE_MPTR

                // Phase 1
                for {
                    let proof_cptr_end := add(proof_cptr, 0x04c0)
                } lt(proof_cptr, proof_cptr_end) {

                } {
                    success, proof_cptr, hash_mptr := read_ec_point(
                        success,
                        proof_cptr,
                        hash_mptr,
                        q
                    )
                }

                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                )

                // Phase 2
                for {
                    let proof_cptr_end := add(proof_cptr, 0x0480)
                } lt(proof_cptr, proof_cptr_end) {

                } {
                    success, proof_cptr, hash_mptr := read_ec_point(
                        success,
                        proof_cptr,
                        hash_mptr,
                        q
                    )
                }

                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                )
                challenge_mptr := squeeze_challenge_cont(challenge_mptr, r)

                // Phase 3
                for {
                    let proof_cptr_end := add(proof_cptr, 0x0340)
                } lt(proof_cptr, proof_cptr_end) {

                } {
                    success, proof_cptr, hash_mptr := read_ec_point(
                        success,
                        proof_cptr,
                        hash_mptr,
                        q
                    )
                }

                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                )

                // Phase 4
                for {
                    let proof_cptr_end := add(proof_cptr, 0x0140)
                } lt(proof_cptr, proof_cptr_end) {

                } {
                    success, proof_cptr, hash_mptr := read_ec_point(
                        success,
                        proof_cptr,
                        hash_mptr,
                        q
                    )
                }

                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                )

                // Read evaluations
                for {
                    let proof_cptr_end := add(proof_cptr, 0x0d40)
                } lt(proof_cptr, proof_cptr_end) {

                } {
                    let eval := calldataload(proof_cptr)
                    success := and(success, lt(eval, r))
                    mstore(hash_mptr, eval)
                    proof_cptr := add(proof_cptr, 0x20)
                    hash_mptr := add(hash_mptr, 0x20)
                }

                // Read batch opening proof and generate challenges
                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                ) // zeta
                challenge_mptr := squeeze_challenge_cont(challenge_mptr, r) // nu

                success, proof_cptr, hash_mptr := read_ec_point(
                    success,
                    proof_cptr,
                    hash_mptr,
                    q
                ) // W

                challenge_mptr, hash_mptr := squeeze_challenge(
                    challenge_mptr,
                    hash_mptr,
                    r
                ) // mu

                success, proof_cptr, hash_mptr := read_ec_point(
                    success,
                    proof_cptr,
                    hash_mptr,
                    q
                ) // W'

                // Copy full vk into memory
                extcodecopy(vk, VK_MPTR, 0x00, 0x0860)

                // Read accumulator from instances
                if mload(HAS_ACCUMULATOR_MPTR) {
                    let num_limbs := mload(NUM_ACC_LIMBS_MPTR)
                    let num_limb_bits := mload(NUM_ACC_LIMB_BITS_MPTR)

                    let cptr := add(
                        INSTANCE_CPTR,
                        mul(mload(ACC_OFFSET_MPTR), 0x20)
                    )
                    let lhs_y_off := mul(num_limbs, 0x20)
                    let rhs_x_off := mul(lhs_y_off, 2)
                    let rhs_y_off := mul(lhs_y_off, 3)
                    let lhs_x := calldataload(cptr)
                    let lhs_y := calldataload(add(cptr, lhs_y_off))
                    let rhs_x := calldataload(add(cptr, rhs_x_off))
                    let rhs_y := calldataload(add(cptr, rhs_y_off))
                    for {
                        let cptr_end := add(cptr, mul(0x20, num_limbs))
                        let shift := num_limb_bits
                    } lt(cptr, cptr_end) {

                    } {
                        cptr := add(cptr, 0x20)
                        lhs_x := add(lhs_x, shl(shift, calldataload(cptr)))
                        lhs_y := add(
                            lhs_y,
                            shl(shift, calldataload(add(cptr, lhs_y_off)))
                        )
                        rhs_x := add(
                            rhs_x,
                            shl(shift, calldataload(add(cptr, rhs_x_off)))
                        )
                        rhs_y := add(
                            rhs_y,
                            shl(shift, calldataload(add(cptr, rhs_y_off)))
                        )
                        shift := add(shift, num_limb_bits)
                    }

                    success := and(
                        success,
                        eq(
                            mulmod(lhs_y, lhs_y, q),
                            addmod(
                                mulmod(lhs_x, mulmod(lhs_x, lhs_x, q), q),
                                3,
                                q
                            )
                        )
                    )
                    success := and(
                        success,
                        eq(
                            mulmod(rhs_y, rhs_y, q),
                            addmod(
                                mulmod(rhs_x, mulmod(rhs_x, rhs_x, q), q),
                                3,
                                q
                            )
                        )
                    )

                    mstore(ACC_LHS_X_MPTR, lhs_x)
                    mstore(ACC_LHS_Y_MPTR, lhs_y)
                    mstore(ACC_RHS_X_MPTR, rhs_x)
                    mstore(ACC_RHS_Y_MPTR, rhs_y)
                }

                pop(q)
            }

            // Revert earlier if anything from calldata is invalid
            if iszero(success) {
                revert(0, 0)
            }

            // Compute lagrange evaluations and instance evaluation
            {
                let k := mload(K_MPTR)
                let x := mload(X_MPTR)
                let x_n := x
                for {
                    let idx := 0
                } lt(idx, k) {
                    idx := add(idx, 1)
                } {
                    x_n := mulmod(x_n, x_n, r)
                }

                let omega := mload(OMEGA_MPTR)

                let mptr := X_N_MPTR
                let mptr_end := add(
                    mptr,
                    mul(0x20, add(mload(NUM_INSTANCES_MPTR), 6))
                )
                if iszero(mload(NUM_INSTANCES_MPTR)) {
                    mptr_end := add(mptr_end, 0x20)
                }
                for {
                    let pow_of_omega := mload(OMEGA_INV_TO_L_MPTR)
                } lt(mptr, mptr_end) {
                    mptr := add(mptr, 0x20)
                } {
                    mstore(mptr, addmod(x, sub(r, pow_of_omega), r))
                    pow_of_omega := mulmod(pow_of_omega, omega, r)
                }
                let x_n_minus_1 := addmod(x_n, sub(r, 1), r)
                mstore(mptr_end, x_n_minus_1)
                success := batch_invert(
                    success,
                    X_N_MPTR,
                    add(mptr_end, 0x20),
                    r
                )

                mptr := X_N_MPTR
                let l_i_common := mulmod(x_n_minus_1, mload(N_INV_MPTR), r)
                for {
                    let pow_of_omega := mload(OMEGA_INV_TO_L_MPTR)
                } lt(mptr, mptr_end) {
                    mptr := add(mptr, 0x20)
                } {
                    mstore(
                        mptr,
                        mulmod(
                            l_i_common,
                            mulmod(mload(mptr), pow_of_omega, r),
                            r
                        )
                    )
                    pow_of_omega := mulmod(pow_of_omega, omega, r)
                }

                let l_blind := mload(add(X_N_MPTR, 0x20))
                let l_i_cptr := add(X_N_MPTR, 0x40)
                for {
                    let l_i_cptr_end := add(X_N_MPTR, 0xc0)
                } lt(l_i_cptr, l_i_cptr_end) {
                    l_i_cptr := add(l_i_cptr, 0x20)
                } {
                    l_blind := addmod(l_blind, mload(l_i_cptr), r)
                }

                let instance_eval := 0
                for {
                    let instance_cptr := INSTANCE_CPTR
                    let instance_cptr_end := add(
                        instance_cptr,
                        mul(0x20, mload(NUM_INSTANCES_MPTR))
                    )
                } lt(instance_cptr, instance_cptr_end) {
                    instance_cptr := add(instance_cptr, 0x20)
                    l_i_cptr := add(l_i_cptr, 0x20)
                } {
                    instance_eval := addmod(
                        instance_eval,
                        mulmod(mload(l_i_cptr), calldataload(instance_cptr), r),
                        r
                    )
                }

                let x_n_minus_1_inv := mload(mptr_end)
                let l_last := mload(X_N_MPTR)
                let l_0 := mload(add(X_N_MPTR, 0xc0))

                mstore(X_N_MPTR, x_n)
                mstore(X_N_MINUS_1_INV_MPTR, x_n_minus_1_inv)
                mstore(L_LAST_MPTR, l_last)
                mstore(L_BLIND_MPTR, l_blind)
                mstore(L_0_MPTR, l_0)
                mstore(INSTANCE_EVAL_MPTR, instance_eval)
            }

            // Compute quotient evavluation
            {
                let quotient_eval_numer
                let
                    delta
                := 4131629893567559867359510883348571134090853742863529169391034518566172092834
                let y := mload(Y_MPTR)
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x2
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x3
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_3 := calldataload(0x0ec4)
                    let var7 := 0x0
                    let a_5 := calldataload(0x0ee4)
                    let var8 := 0x1
                    let var9 := mulmod(a_5, var8, r)
                    let var10 := addmod(var7, var9, r)
                    let a_6 := calldataload(0x0f04)
                    let var11 := 0x10000
                    let var12 := mulmod(a_6, var11, r)
                    let var13 := addmod(var10, var12, r)
                    let a_7 := calldataload(0x0f24)
                    let var14 := 0x100000000
                    let var15 := mulmod(a_7, var14, r)
                    let var16 := addmod(var13, var15, r)
                    let a_8 := calldataload(0x0f44)
                    let var17 := 0x1000000000000
                    let var18 := mulmod(a_8, var17, r)
                    let var19 := addmod(var16, var18, r)
                    let a_9 := calldataload(0x0f64)
                    let
                        var20
                    := 0x0000000000000000000000000000000000000000000000010000000000000000
                    let var21 := mulmod(a_9, var20, r)
                    let var22 := addmod(var19, var21, r)
                    let var23 := sub(r, var22)
                    let var24 := addmod(a_3, var23, r)
                    let var25 := mulmod(var6, var24, r)
                    quotient_eval_numer := var25
                }
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x2
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x3
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_4 := calldataload(0x0ea4)
                    let var7 := 0x0
                    let a_10 := calldataload(0x0f84)
                    let var8 := 0x1
                    let var9 := mulmod(a_10, var8, r)
                    let var10 := addmod(var7, var9, r)
                    let a_11 := calldataload(0x0fa4)
                    let var11 := 0x10000
                    let var12 := mulmod(a_11, var11, r)
                    let var13 := addmod(var10, var12, r)
                    let a_12 := calldataload(0x0fc4)
                    let var14 := 0x100000000
                    let var15 := mulmod(a_12, var14, r)
                    let var16 := addmod(var13, var15, r)
                    let a_13 := calldataload(0x0fe4)
                    let var17 := 0x1000000000000
                    let var18 := mulmod(a_13, var17, r)
                    let var19 := addmod(var16, var18, r)
                    let var20 := sub(r, var19)
                    let var21 := addmod(a_4, var20, r)
                    let var22 := mulmod(var6, var21, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var22,
                        r
                    )
                }
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x2
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x3
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_3 := calldataload(0x0ec4)
                    let
                        var7
                    := 0x000000000000000000000000000000000000000000000000ffffffff00000001
                    let var8 := sub(r, var7)
                    let var9 := addmod(a_3, var8, r)
                    let a_4 := calldataload(0x0ea4)
                    let var10 := addmod(var9, a_4, r)
                    let var11 := mulmod(var6, var10, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var11,
                        r
                    )
                }
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x1
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x3
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_0 := calldataload(0x0e44)
                    let a_1 := calldataload(0x0e64)
                    let var7 := mulmod(a_0, a_1, r)
                    let a_2 := calldataload(0x0e84)
                    let var8 := addmod(var7, a_2, r)
                    let
                        var9
                    := 0x000000000000000000000000000000000000000000000000ffffffff00000001
                    let a_3 := calldataload(0x0ec4)
                    let var10 := mulmod(var9, a_3, r)
                    let var11 := sub(r, var10)
                    let var12 := addmod(var8, var11, r)
                    let a_4 := calldataload(0x0ea4)
                    let var13 := sub(r, a_4)
                    let var14 := addmod(var12, var13, r)
                    let var15 := mulmod(var6, var14, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var15,
                        r
                    )
                }
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x1
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x2
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_0 := calldataload(0x0e44)
                    let a_1 := calldataload(0x0e64)
                    let var7 := mulmod(a_0, a_1, r)
                    let var8 := 0x7
                    let a_0_next_1 := calldataload(0x1004)
                    let var9 := mulmod(var8, a_0_next_1, r)
                    let a_1_next_1 := calldataload(0x1024)
                    let var10 := mulmod(var9, a_1_next_1, r)
                    let var11 := addmod(var7, var10, r)
                    let a_2 := calldataload(0x0e84)
                    let var12 := addmod(var11, a_2, r)
                    let
                        var13
                    := 0x000000000000000000000000000000000000000000000000ffffffff00000001
                    let a_3 := calldataload(0x0ec4)
                    let var14 := mulmod(var13, a_3, r)
                    let a_4 := calldataload(0x0ea4)
                    let var15 := addmod(var14, a_4, r)
                    let var16 := sub(r, var15)
                    let var17 := addmod(var12, var16, r)
                    let var18 := mulmod(var6, var17, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var18,
                        r
                    )
                }
                {
                    let f_8 := calldataload(0x12e4)
                    let var0 := 0x1
                    let var1 := sub(r, f_8)
                    let var2 := addmod(var0, var1, r)
                    let var3 := mulmod(f_8, var2, r)
                    let var4 := 0x2
                    let var5 := addmod(var4, var1, r)
                    let var6 := mulmod(var3, var5, r)
                    let a_0 := calldataload(0x0e44)
                    let a_1_next_1 := calldataload(0x1024)
                    let var7 := mulmod(a_0, a_1_next_1, r)
                    let a_0_next_1 := calldataload(0x1004)
                    let a_1 := calldataload(0x0e64)
                    let var8 := mulmod(a_0_next_1, a_1, r)
                    let var9 := addmod(var7, var8, r)
                    let a_2_next_1 := calldataload(0x1044)
                    let var10 := addmod(var9, a_2_next_1, r)
                    let
                        var11
                    := 0x000000000000000000000000000000000000000000000000ffffffff00000001
                    let a_3_next_1 := calldataload(0x1064)
                    let var12 := mulmod(var11, a_3_next_1, r)
                    let a_4_next_1 := calldataload(0x1084)
                    let var13 := addmod(var12, a_4_next_1, r)
                    let var14 := sub(r, var13)
                    let var15 := addmod(var10, var14, r)
                    let var16 := mulmod(var6, var15, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var16,
                        r
                    )
                }
                {
                    let f_10 := calldataload(0x1324)
                    let a_14_next_1 := calldataload(0x1144)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x251e7fdf99591080080b0af133b9e4369f22e57ace3cd7f64fc6fdbcf38d7da1
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let
                        var10
                    := 0x25fb50b65acf4fb047cbd3b1c17d97c7fe26ea9ca238d6e348550486e91c7765
                    let var11 := mulmod(var9, var10, r)
                    let var12 := addmod(var8, var11, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var13 := addmod(a_16, f_4, r)
                    let
                        var14
                    := 0x293d617d7da72102355f39ebf62f91b06deb5325f367a4556ea1e31ed5767833
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var12, var15, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var17 := addmod(a_17, f_5, r)
                    let
                        var18
                    := 0x104d0295ab00c85e960111ac25da474366599e575a9b7edf6145f14ba6d3c1c4
                    let var19 := mulmod(var17, var18, r)
                    let var20 := addmod(var16, var19, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var21 := addmod(a_18, f_6, r)
                    let
                        var22
                    := 0x0aaa35e2c84baf117dea3e336cd96a39792b3813954fe9bf3ed5b90f2f69c977
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var20, var23, r)
                    let var25 := sub(r, var24)
                    let var26 := addmod(a_14_next_1, var25, r)
                    let var27 := mulmod(f_10, var26, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var27,
                        r
                    )
                }
                {
                    let f_10 := calldataload(0x1324)
                    let a_15_next_1 := calldataload(0x1164)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2a70b9f1d4bbccdbc03e17c1d1dcdb02052903dc6609ea6969f661b2eb74c839
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let
                        var10
                    := 0x281154651c921e746315a9934f1b8a1bba9f92ad8ef4b979115b8e2e991ccd7a
                    let var11 := mulmod(var9, var10, r)
                    let var12 := addmod(var8, var11, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var13 := addmod(a_16, f_4, r)
                    let
                        var14
                    := 0x28c2be2f8264f95f0b53c732134efa338ccd8fdb9ee2b45fb86a894f7db36c37
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var12, var15, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var17 := addmod(a_17, f_5, r)
                    let
                        var18
                    := 0x21888041e6febd546d427c890b1883bb9b626d8cb4dc18dcc4ec8fa75e530a13
                    let var19 := mulmod(var17, var18, r)
                    let var20 := addmod(var16, var19, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var21 := addmod(a_18, f_6, r)
                    let
                        var22
                    := 0x14ddb5fada0171db80195b9592d8cf2be810930e3ea4574a350d65e2cbff4941
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var20, var23, r)
                    let var25 := sub(r, var24)
                    let var26 := addmod(a_15_next_1, var25, r)
                    let var27 := mulmod(f_10, var26, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var27,
                        r
                    )
                }
                {
                    let f_10 := calldataload(0x1324)
                    let a_16_next_1 := calldataload(0x1184)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2f69a7198e1fbcc7dea43265306a37ed55b91bff652ad69aa4fa8478970d401d
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let
                        var10
                    := 0x001c1edd62645b73ad931ab80e37bbb267ba312b34140e716d6a3747594d3052
                    let var11 := mulmod(var9, var10, r)
                    let var12 := addmod(var8, var11, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var13 := addmod(a_16, f_4, r)
                    let
                        var14
                    := 0x15b98ce93e47bc64ce2f2c96c69663c439c40c603049466fa7f9a4b228bfc32b
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var12, var15, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var17 := addmod(a_17, f_5, r)
                    let
                        var18
                    := 0x12c7e2adfa524e5958f65be2fbac809fcba8458b28e44d9265051de33163cf9c
                    let var19 := mulmod(var17, var18, r)
                    let var20 := addmod(var16, var19, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var21 := addmod(a_18, f_6, r)
                    let
                        var22
                    := 0x2efc2b90d688134849018222e7b8922eaf67ce79816ef468531ec2de53bbd167
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var20, var23, r)
                    let var25 := sub(r, var24)
                    let var26 := addmod(a_16_next_1, var25, r)
                    let var27 := mulmod(f_10, var26, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var27,
                        r
                    )
                }
                {
                    let f_10 := calldataload(0x1324)
                    let a_17_next_1 := calldataload(0x11a4)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x0c3f050a6bf5af151981e55e3e1a29a13c3ffa4550bd2514f1afd6c5f721f830
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let
                        var10
                    := 0x0dec54e6dbf75205fa75ba7992bd34f08b2efe2ecd424a73eda7784320a1a36e
                    let var11 := mulmod(var9, var10, r)
                    let var12 := addmod(var8, var11, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var13 := addmod(a_16, f_4, r)
                    let
                        var14
                    := 0x1c482a25a729f5df20225815034b196098364a11f4d988fb7cc75cf32d8136fa
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var12, var15, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var17 := addmod(a_17, f_5, r)
                    let
                        var18
                    := 0x2625ce48a7b39a4252732624e4ab94360812ac2fc9a14a5fb8b607ae9fd8514a
                    let var19 := mulmod(var17, var18, r)
                    let var20 := addmod(var16, var19, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var21 := addmod(a_18, f_6, r)
                    let
                        var22
                    := 0x07f017a7ebd56dd086f7cd4fd710c509ed7ef8e300b9a8bb9fb9f28af710251f
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var20, var23, r)
                    let var25 := sub(r, var24)
                    let var26 := addmod(a_17_next_1, var25, r)
                    let var27 := mulmod(f_10, var26, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var27,
                        r
                    )
                }
                {
                    let f_10 := calldataload(0x1324)
                    let a_18_next_1 := calldataload(0x11c4)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2a20e3a4a0e57d92f97c9d6186c6c3ea7c5e55c20146259be2f78c2ccc2e3595
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let
                        var10
                    := 0x1049f8210566b51faafb1e9a5d63c0ee701673aed820d9c4403b01feb727a549
                    let var11 := mulmod(var9, var10, r)
                    let var12 := addmod(var8, var11, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var13 := addmod(a_16, f_4, r)
                    let
                        var14
                    := 0x02ecac687ef5b4b568002bd9d1b96b4bef357a69e3e86b5561b9299b82d69c8e
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var12, var15, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var17 := addmod(a_17, f_5, r)
                    let
                        var18
                    := 0x2d3a1aea2e6d44466808f88c9ba903d3bdcb6b58ba40441ed4ebcf11bbe1e37b
                    let var19 := mulmod(var17, var18, r)
                    let var20 := addmod(var16, var19, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var21 := addmod(a_18, f_6, r)
                    let
                        var22
                    := 0x14074bb14c982c81c9ad171e4f35fe49b39c4a7a72dbb6d9c98d803bfed65e64
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var20, var23, r)
                    let var25 := sub(r, var24)
                    let var26 := addmod(a_18_next_1, var25, r)
                    let var27 := mulmod(f_10, var26, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var27,
                        r
                    )
                }
                {
                    let f_9 := calldataload(0x1304)
                    let a_14_next_1 := calldataload(0x1144)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x251e7fdf99591080080b0af133b9e4369f22e57ace3cd7f64fc6fdbcf38d7da1
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let var10 := mulmod(var9, var9, r)
                    let var11 := mulmod(var10, var9, r)
                    let var12 := mulmod(var11, var9, r)
                    let var13 := mulmod(var12, var9, r)
                    let
                        var14
                    := 0x25fb50b65acf4fb047cbd3b1c17d97c7fe26ea9ca238d6e348550486e91c7765
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var8, var15, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var17 := addmod(a_16, f_4, r)
                    let var18 := mulmod(var17, var17, r)
                    let var19 := mulmod(var18, var17, r)
                    let var20 := mulmod(var19, var17, r)
                    let var21 := mulmod(var20, var17, r)
                    let
                        var22
                    := 0x293d617d7da72102355f39ebf62f91b06deb5325f367a4556ea1e31ed5767833
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var16, var23, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var25 := addmod(a_17, f_5, r)
                    let var26 := mulmod(var25, var25, r)
                    let var27 := mulmod(var26, var25, r)
                    let var28 := mulmod(var27, var25, r)
                    let var29 := mulmod(var28, var25, r)
                    let
                        var30
                    := 0x104d0295ab00c85e960111ac25da474366599e575a9b7edf6145f14ba6d3c1c4
                    let var31 := mulmod(var29, var30, r)
                    let var32 := addmod(var24, var31, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var33 := addmod(a_18, f_6, r)
                    let var34 := mulmod(var33, var33, r)
                    let var35 := mulmod(var34, var33, r)
                    let var36 := mulmod(var35, var33, r)
                    let var37 := mulmod(var36, var33, r)
                    let
                        var38
                    := 0x0aaa35e2c84baf117dea3e336cd96a39792b3813954fe9bf3ed5b90f2f69c977
                    let var39 := mulmod(var37, var38, r)
                    let var40 := addmod(var32, var39, r)
                    let var41 := sub(r, var40)
                    let var42 := addmod(a_14_next_1, var41, r)
                    let var43 := mulmod(f_9, var42, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var43,
                        r
                    )
                }
                {
                    let f_9 := calldataload(0x1304)
                    let a_15_next_1 := calldataload(0x1164)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2a70b9f1d4bbccdbc03e17c1d1dcdb02052903dc6609ea6969f661b2eb74c839
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let var10 := mulmod(var9, var9, r)
                    let var11 := mulmod(var10, var9, r)
                    let var12 := mulmod(var11, var9, r)
                    let var13 := mulmod(var12, var9, r)
                    let
                        var14
                    := 0x281154651c921e746315a9934f1b8a1bba9f92ad8ef4b979115b8e2e991ccd7a
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var8, var15, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var17 := addmod(a_16, f_4, r)
                    let var18 := mulmod(var17, var17, r)
                    let var19 := mulmod(var18, var17, r)
                    let var20 := mulmod(var19, var17, r)
                    let var21 := mulmod(var20, var17, r)
                    let
                        var22
                    := 0x28c2be2f8264f95f0b53c732134efa338ccd8fdb9ee2b45fb86a894f7db36c37
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var16, var23, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var25 := addmod(a_17, f_5, r)
                    let var26 := mulmod(var25, var25, r)
                    let var27 := mulmod(var26, var25, r)
                    let var28 := mulmod(var27, var25, r)
                    let var29 := mulmod(var28, var25, r)
                    let
                        var30
                    := 0x21888041e6febd546d427c890b1883bb9b626d8cb4dc18dcc4ec8fa75e530a13
                    let var31 := mulmod(var29, var30, r)
                    let var32 := addmod(var24, var31, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var33 := addmod(a_18, f_6, r)
                    let var34 := mulmod(var33, var33, r)
                    let var35 := mulmod(var34, var33, r)
                    let var36 := mulmod(var35, var33, r)
                    let var37 := mulmod(var36, var33, r)
                    let
                        var38
                    := 0x14ddb5fada0171db80195b9592d8cf2be810930e3ea4574a350d65e2cbff4941
                    let var39 := mulmod(var37, var38, r)
                    let var40 := addmod(var32, var39, r)
                    let var41 := sub(r, var40)
                    let var42 := addmod(a_15_next_1, var41, r)
                    let var43 := mulmod(f_9, var42, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var43,
                        r
                    )
                }
                {
                    let f_9 := calldataload(0x1304)
                    let a_16_next_1 := calldataload(0x1184)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2f69a7198e1fbcc7dea43265306a37ed55b91bff652ad69aa4fa8478970d401d
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let var10 := mulmod(var9, var9, r)
                    let var11 := mulmod(var10, var9, r)
                    let var12 := mulmod(var11, var9, r)
                    let var13 := mulmod(var12, var9, r)
                    let
                        var14
                    := 0x001c1edd62645b73ad931ab80e37bbb267ba312b34140e716d6a3747594d3052
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var8, var15, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var17 := addmod(a_16, f_4, r)
                    let var18 := mulmod(var17, var17, r)
                    let var19 := mulmod(var18, var17, r)
                    let var20 := mulmod(var19, var17, r)
                    let var21 := mulmod(var20, var17, r)
                    let
                        var22
                    := 0x15b98ce93e47bc64ce2f2c96c69663c439c40c603049466fa7f9a4b228bfc32b
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var16, var23, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var25 := addmod(a_17, f_5, r)
                    let var26 := mulmod(var25, var25, r)
                    let var27 := mulmod(var26, var25, r)
                    let var28 := mulmod(var27, var25, r)
                    let var29 := mulmod(var28, var25, r)
                    let
                        var30
                    := 0x12c7e2adfa524e5958f65be2fbac809fcba8458b28e44d9265051de33163cf9c
                    let var31 := mulmod(var29, var30, r)
                    let var32 := addmod(var24, var31, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var33 := addmod(a_18, f_6, r)
                    let var34 := mulmod(var33, var33, r)
                    let var35 := mulmod(var34, var33, r)
                    let var36 := mulmod(var35, var33, r)
                    let var37 := mulmod(var36, var33, r)
                    let
                        var38
                    := 0x2efc2b90d688134849018222e7b8922eaf67ce79816ef468531ec2de53bbd167
                    let var39 := mulmod(var37, var38, r)
                    let var40 := addmod(var32, var39, r)
                    let var41 := sub(r, var40)
                    let var42 := addmod(a_16_next_1, var41, r)
                    let var43 := mulmod(f_9, var42, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var43,
                        r
                    )
                }
                {
                    let f_9 := calldataload(0x1304)
                    let a_17_next_1 := calldataload(0x11a4)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x0c3f050a6bf5af151981e55e3e1a29a13c3ffa4550bd2514f1afd6c5f721f830
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let var10 := mulmod(var9, var9, r)
                    let var11 := mulmod(var10, var9, r)
                    let var12 := mulmod(var11, var9, r)
                    let var13 := mulmod(var12, var9, r)
                    let
                        var14
                    := 0x0dec54e6dbf75205fa75ba7992bd34f08b2efe2ecd424a73eda7784320a1a36e
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var8, var15, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var17 := addmod(a_16, f_4, r)
                    let var18 := mulmod(var17, var17, r)
                    let var19 := mulmod(var18, var17, r)
                    let var20 := mulmod(var19, var17, r)
                    let var21 := mulmod(var20, var17, r)
                    let
                        var22
                    := 0x1c482a25a729f5df20225815034b196098364a11f4d988fb7cc75cf32d8136fa
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var16, var23, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var25 := addmod(a_17, f_5, r)
                    let var26 := mulmod(var25, var25, r)
                    let var27 := mulmod(var26, var25, r)
                    let var28 := mulmod(var27, var25, r)
                    let var29 := mulmod(var28, var25, r)
                    let
                        var30
                    := 0x2625ce48a7b39a4252732624e4ab94360812ac2fc9a14a5fb8b607ae9fd8514a
                    let var31 := mulmod(var29, var30, r)
                    let var32 := addmod(var24, var31, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var33 := addmod(a_18, f_6, r)
                    let var34 := mulmod(var33, var33, r)
                    let var35 := mulmod(var34, var33, r)
                    let var36 := mulmod(var35, var33, r)
                    let var37 := mulmod(var36, var33, r)
                    let
                        var38
                    := 0x07f017a7ebd56dd086f7cd4fd710c509ed7ef8e300b9a8bb9fb9f28af710251f
                    let var39 := mulmod(var37, var38, r)
                    let var40 := addmod(var32, var39, r)
                    let var41 := sub(r, var40)
                    let var42 := addmod(a_17_next_1, var41, r)
                    let var43 := mulmod(f_9, var42, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var43,
                        r
                    )
                }
                {
                    let f_9 := calldataload(0x1304)
                    let a_18_next_1 := calldataload(0x11c4)
                    let var0 := 0x0
                    let a_14 := calldataload(0x10a4)
                    let f_2 := calldataload(0x1224)
                    let var1 := addmod(a_14, f_2, r)
                    let var2 := mulmod(var1, var1, r)
                    let var3 := mulmod(var2, var1, r)
                    let var4 := mulmod(var3, var1, r)
                    let var5 := mulmod(var4, var1, r)
                    let
                        var6
                    := 0x2a20e3a4a0e57d92f97c9d6186c6c3ea7c5e55c20146259be2f78c2ccc2e3595
                    let var7 := mulmod(var5, var6, r)
                    let var8 := addmod(var0, var7, r)
                    let a_15 := calldataload(0x10c4)
                    let f_3 := calldataload(0x1244)
                    let var9 := addmod(a_15, f_3, r)
                    let var10 := mulmod(var9, var9, r)
                    let var11 := mulmod(var10, var9, r)
                    let var12 := mulmod(var11, var9, r)
                    let var13 := mulmod(var12, var9, r)
                    let
                        var14
                    := 0x1049f8210566b51faafb1e9a5d63c0ee701673aed820d9c4403b01feb727a549
                    let var15 := mulmod(var13, var14, r)
                    let var16 := addmod(var8, var15, r)
                    let a_16 := calldataload(0x10e4)
                    let f_4 := calldataload(0x1264)
                    let var17 := addmod(a_16, f_4, r)
                    let var18 := mulmod(var17, var17, r)
                    let var19 := mulmod(var18, var17, r)
                    let var20 := mulmod(var19, var17, r)
                    let var21 := mulmod(var20, var17, r)
                    let
                        var22
                    := 0x02ecac687ef5b4b568002bd9d1b96b4bef357a69e3e86b5561b9299b82d69c8e
                    let var23 := mulmod(var21, var22, r)
                    let var24 := addmod(var16, var23, r)
                    let a_17 := calldataload(0x1104)
                    let f_5 := calldataload(0x1284)
                    let var25 := addmod(a_17, f_5, r)
                    let var26 := mulmod(var25, var25, r)
                    let var27 := mulmod(var26, var25, r)
                    let var28 := mulmod(var27, var25, r)
                    let var29 := mulmod(var28, var25, r)
                    let
                        var30
                    := 0x2d3a1aea2e6d44466808f88c9ba903d3bdcb6b58ba40441ed4ebcf11bbe1e37b
                    let var31 := mulmod(var29, var30, r)
                    let var32 := addmod(var24, var31, r)
                    let a_18 := calldataload(0x1124)
                    let f_6 := calldataload(0x12a4)
                    let var33 := addmod(a_18, f_6, r)
                    let var34 := mulmod(var33, var33, r)
                    let var35 := mulmod(var34, var33, r)
                    let var36 := mulmod(var35, var33, r)
                    let var37 := mulmod(var36, var33, r)
                    let
                        var38
                    := 0x14074bb14c982c81c9ad171e4f35fe49b39c4a7a72dbb6d9c98d803bfed65e64
                    let var39 := mulmod(var37, var38, r)
                    let var40 := addmod(var32, var39, r)
                    let var41 := sub(r, var40)
                    let var42 := addmod(a_18_next_1, var41, r)
                    let var43 := mulmod(f_9, var42, r)
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        var43,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        sub(r, mulmod(l_0, calldataload(0x14e4), r)),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let perm_z_last := calldataload(0x15a4)
                    let eval := mulmod(
                        mload(L_LAST_MPTR),
                        addmod(
                            mulmod(perm_z_last, perm_z_last, r),
                            sub(r, perm_z_last),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1544),
                            sub(r, calldataload(0x1524)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x15a4),
                            sub(r, calldataload(0x1584)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let gamma := mload(GAMMA_MPTR)
                    let beta := mload(BETA_MPTR)
                    let lhs := calldataload(0x1504)
                    let rhs := calldataload(0x14e4)
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x0e44),
                                mulmod(beta, calldataload(0x1364), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x0e64),
                                mulmod(beta, calldataload(0x1384), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x0e84),
                                mulmod(beta, calldataload(0x13a4), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x0ea4),
                                mulmod(beta, calldataload(0x13c4), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(beta, mload(X_MPTR), r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x0e44), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x0e64), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x0e84), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x0ea4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    let left_sub_right := addmod(lhs, sub(r, rhs), r)
                    let eval := addmod(
                        left_sub_right,
                        sub(
                            r,
                            mulmod(
                                left_sub_right,
                                addmod(
                                    mload(L_LAST_MPTR),
                                    mload(L_BLIND_MPTR),
                                    r
                                ),
                                r
                            )
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let gamma := mload(GAMMA_MPTR)
                    let beta := mload(BETA_MPTR)
                    let lhs := calldataload(0x1564)
                    let rhs := calldataload(0x1544)
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x0ec4),
                                mulmod(beta, calldataload(0x13e4), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                mload(INSTANCE_EVAL_MPTR),
                                mulmod(beta, calldataload(0x1404), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x11e4),
                                mulmod(beta, calldataload(0x1424), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x10a4),
                                mulmod(beta, calldataload(0x1444), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x0ec4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(mload(INSTANCE_EVAL_MPTR), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x11e4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x10a4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    let left_sub_right := addmod(lhs, sub(r, rhs), r)
                    let eval := addmod(
                        left_sub_right,
                        sub(
                            r,
                            mulmod(
                                left_sub_right,
                                addmod(
                                    mload(L_LAST_MPTR),
                                    mload(L_BLIND_MPTR),
                                    r
                                ),
                                r
                            )
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let gamma := mload(GAMMA_MPTR)
                    let beta := mload(BETA_MPTR)
                    let lhs := calldataload(0x15c4)
                    let rhs := calldataload(0x15a4)
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x10c4),
                                mulmod(beta, calldataload(0x1464), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x10e4),
                                mulmod(beta, calldataload(0x1484), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x1104),
                                mulmod(beta, calldataload(0x14a4), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    lhs := mulmod(
                        lhs,
                        addmod(
                            addmod(
                                calldataload(0x1124),
                                mulmod(beta, calldataload(0x14c4), r),
                                r
                            ),
                            gamma,
                            r
                        ),
                        r
                    )
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x10c4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x10e4), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x1104), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    mstore(0x00, mulmod(mload(0x00), delta, r))
                    rhs := mulmod(
                        rhs,
                        addmod(
                            addmod(calldataload(0x1124), mload(0x00), r),
                            gamma,
                            r
                        ),
                        r
                    )
                    let left_sub_right := addmod(lhs, sub(r, rhs), r)
                    let eval := addmod(
                        left_sub_right,
                        sub(
                            r,
                            mulmod(
                                left_sub_right,
                                addmod(
                                    mload(L_LAST_MPTR),
                                    mload(L_BLIND_MPTR),
                                    r
                                ),
                                r
                            )
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x15e4)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x15e4),
                                calldataload(0x15e4),
                                r
                            ),
                            sub(r, calldataload(0x15e4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_5 := calldataload(0x0ee4)
                        input := a_5
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1604),
                        mulmod(
                            addmod(calldataload(0x1624), beta, r),
                            addmod(calldataload(0x1664), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x15e4),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1624),
                            sub(r, calldataload(0x1664)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1624),
                                sub(r, calldataload(0x1664)),
                                r
                            ),
                            addmod(
                                calldataload(0x1624),
                                sub(r, calldataload(0x1644)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1684)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1684),
                                calldataload(0x1684),
                                r
                            ),
                            sub(r, calldataload(0x1684)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_6 := calldataload(0x0f04)
                        input := a_6
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x16a4),
                        mulmod(
                            addmod(calldataload(0x16c4), beta, r),
                            addmod(calldataload(0x1704), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1684),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x16c4),
                            sub(r, calldataload(0x1704)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x16c4),
                                sub(r, calldataload(0x1704)),
                                r
                            ),
                            addmod(
                                calldataload(0x16c4),
                                sub(r, calldataload(0x16e4)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1724)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1724),
                                calldataload(0x1724),
                                r
                            ),
                            sub(r, calldataload(0x1724)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_7 := calldataload(0x0f24)
                        input := a_7
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1744),
                        mulmod(
                            addmod(calldataload(0x1764), beta, r),
                            addmod(calldataload(0x17a4), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1724),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1764),
                            sub(r, calldataload(0x17a4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1764),
                                sub(r, calldataload(0x17a4)),
                                r
                            ),
                            addmod(
                                calldataload(0x1764),
                                sub(r, calldataload(0x1784)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x17c4)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x17c4),
                                calldataload(0x17c4),
                                r
                            ),
                            sub(r, calldataload(0x17c4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_8 := calldataload(0x0f44)
                        input := a_8
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x17e4),
                        mulmod(
                            addmod(calldataload(0x1804), beta, r),
                            addmod(calldataload(0x1844), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x17c4),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1804),
                            sub(r, calldataload(0x1844)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1804),
                                sub(r, calldataload(0x1844)),
                                r
                            ),
                            addmod(
                                calldataload(0x1804),
                                sub(r, calldataload(0x1824)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1864)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1864),
                                calldataload(0x1864),
                                r
                            ),
                            sub(r, calldataload(0x1864)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_9 := calldataload(0x0f64)
                        input := a_9
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1884),
                        mulmod(
                            addmod(calldataload(0x18a4), beta, r),
                            addmod(calldataload(0x18e4), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1864),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x18a4),
                            sub(r, calldataload(0x18e4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x18a4),
                                sub(r, calldataload(0x18e4)),
                                r
                            ),
                            addmod(
                                calldataload(0x18a4),
                                sub(r, calldataload(0x18c4)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1904)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1904),
                                calldataload(0x1904),
                                r
                            ),
                            sub(r, calldataload(0x1904)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_10 := calldataload(0x0f84)
                        input := a_10
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1924),
                        mulmod(
                            addmod(calldataload(0x1944), beta, r),
                            addmod(calldataload(0x1984), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1904),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1944),
                            sub(r, calldataload(0x1984)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1944),
                                sub(r, calldataload(0x1984)),
                                r
                            ),
                            addmod(
                                calldataload(0x1944),
                                sub(r, calldataload(0x1964)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x19a4)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x19a4),
                                calldataload(0x19a4),
                                r
                            ),
                            sub(r, calldataload(0x19a4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_11 := calldataload(0x0fa4)
                        input := a_11
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x19c4),
                        mulmod(
                            addmod(calldataload(0x19e4), beta, r),
                            addmod(calldataload(0x1a24), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x19a4),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x19e4),
                            sub(r, calldataload(0x1a24)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x19e4),
                                sub(r, calldataload(0x1a24)),
                                r
                            ),
                            addmod(
                                calldataload(0x19e4),
                                sub(r, calldataload(0x1a04)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1a44)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1a44),
                                calldataload(0x1a44),
                                r
                            ),
                            sub(r, calldataload(0x1a44)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_12 := calldataload(0x0fc4)
                        input := a_12
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1a64),
                        mulmod(
                            addmod(calldataload(0x1a84), beta, r),
                            addmod(calldataload(0x1ac4), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1a44),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1a84),
                            sub(r, calldataload(0x1ac4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1a84),
                                sub(r, calldataload(0x1ac4)),
                                r
                            ),
                            addmod(
                                calldataload(0x1a84),
                                sub(r, calldataload(0x1aa4)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_0 := mload(L_0_MPTR)
                    let eval := addmod(
                        l_0,
                        mulmod(l_0, sub(r, calldataload(0x1ae4)), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let l_last := mload(L_LAST_MPTR)
                    let eval := mulmod(
                        l_last,
                        addmod(
                            mulmod(
                                calldataload(0x1ae4),
                                calldataload(0x1ae4),
                                r
                            ),
                            sub(r, calldataload(0x1ae4)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let theta := mload(THETA_MPTR)
                    let input
                    {
                        let a_13 := calldataload(0x0fe4)
                        input := a_13
                    }
                    let table
                    {
                        let f_1 := calldataload(0x1204)
                        table := f_1
                    }
                    let beta := mload(BETA_MPTR)
                    let gamma := mload(GAMMA_MPTR)
                    let lhs := mulmod(
                        calldataload(0x1b04),
                        mulmod(
                            addmod(calldataload(0x1b24), beta, r),
                            addmod(calldataload(0x1b64), gamma, r),
                            r
                        ),
                        r
                    )
                    let rhs := mulmod(
                        calldataload(0x1ae4),
                        mulmod(
                            addmod(input, beta, r),
                            addmod(table, gamma, r),
                            r
                        ),
                        r
                    )
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        addmod(lhs, sub(r, rhs), r),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        mload(L_0_MPTR),
                        addmod(
                            calldataload(0x1b24),
                            sub(r, calldataload(0x1b64)),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }
                {
                    let eval := mulmod(
                        addmod(
                            1,
                            sub(
                                r,
                                addmod(
                                    mload(L_BLIND_MPTR),
                                    mload(L_LAST_MPTR),
                                    r
                                )
                            ),
                            r
                        ),
                        mulmod(
                            addmod(
                                calldataload(0x1b24),
                                sub(r, calldataload(0x1b64)),
                                r
                            ),
                            addmod(
                                calldataload(0x1b24),
                                sub(r, calldataload(0x1b44)),
                                r
                            ),
                            r
                        ),
                        r
                    )
                    quotient_eval_numer := addmod(
                        mulmod(quotient_eval_numer, y, r),
                        eval,
                        r
                    )
                }

                pop(y)
                pop(delta)

                let quotient_eval := mulmod(
                    quotient_eval_numer,
                    mload(X_N_MINUS_1_INV_MPTR),
                    r
                )
                mstore(QUOTIENT_EVAL_MPTR, quotient_eval)
            }

            // Compute quotient commitment
            {
                mstore(0x00, calldataload(LAST_QUOTIENT_X_CPTR))
                mstore(0x20, calldataload(add(LAST_QUOTIENT_X_CPTR, 0x20)))
                let x_n := mload(X_N_MPTR)
                for {
                    let cptr := sub(LAST_QUOTIENT_X_CPTR, 0x40)
                    let cptr_end := sub(FIRST_QUOTIENT_X_CPTR, 0x40)
                } lt(cptr_end, cptr) {

                } {
                    success := ec_mul_acc(success, x_n)
                    success := ec_add_acc(
                        success,
                        calldataload(cptr),
                        calldataload(add(cptr, 0x20))
                    )
                    cptr := sub(cptr, 0x40)
                }
                mstore(QUOTIENT_X_MPTR, mload(0x00))
                mstore(QUOTIENT_Y_MPTR, mload(0x20))
            }

            // Compute pairing lhs and rhs
            {
                {
                    let x := mload(X_MPTR)
                    let omega := mload(OMEGA_MPTR)
                    let omega_inv := mload(OMEGA_INV_MPTR)
                    let x_pow_of_omega := mulmod(x, omega, r)
                    mstore(0x0360, x_pow_of_omega)
                    mstore(0x0340, x)
                    x_pow_of_omega := mulmod(x, omega_inv, r)
                    mstore(0x0320, x_pow_of_omega)
                    x_pow_of_omega := mulmod(x_pow_of_omega, omega_inv, r)
                    x_pow_of_omega := mulmod(x_pow_of_omega, omega_inv, r)
                    x_pow_of_omega := mulmod(x_pow_of_omega, omega_inv, r)
                    x_pow_of_omega := mulmod(x_pow_of_omega, omega_inv, r)
                    x_pow_of_omega := mulmod(x_pow_of_omega, omega_inv, r)
                    mstore(0x0300, x_pow_of_omega)
                }
                {
                    let mu := mload(MU_MPTR)
                    for {
                        let mptr := 0x0380
                        let mptr_end := 0x0400
                        let point_mptr := 0x0300
                    } lt(mptr, mptr_end) {
                        mptr := add(mptr, 0x20)
                        point_mptr := add(point_mptr, 0x20)
                    } {
                        mstore(mptr, addmod(mu, sub(r, mload(point_mptr)), r))
                    }
                    let s
                    s := mload(0x03c0)
                    s := mulmod(s, mload(0x03e0), r)
                    mstore(0x0400, s)
                    let diff
                    diff := mload(0x0380)
                    diff := mulmod(diff, mload(0x03a0), r)
                    mstore(0x0420, diff)
                    mstore(0x00, diff)
                    diff := mload(0x0380)
                    diff := mulmod(diff, mload(0x03a0), r)
                    diff := mulmod(diff, mload(0x03e0), r)
                    mstore(0x0440, diff)
                    diff := mload(0x03a0)
                    mstore(0x0460, diff)
                    diff := mload(0x0380)
                    diff := mulmod(diff, mload(0x03e0), r)
                    mstore(0x0480, diff)
                }
                {
                    let point_2 := mload(0x0340)
                    let point_3 := mload(0x0360)
                    let coeff
                    coeff := addmod(point_2, sub(r, point_3), r)
                    coeff := mulmod(coeff, mload(0x03c0), r)
                    mstore(0x20, coeff)
                    coeff := addmod(point_3, sub(r, point_2), r)
                    coeff := mulmod(coeff, mload(0x03e0), r)
                    mstore(0x40, coeff)
                }
                {
                    let point_2 := mload(0x0340)
                    let coeff
                    coeff := 1
                    coeff := mulmod(coeff, mload(0x03c0), r)
                    mstore(0x60, coeff)
                }
                {
                    let point_0 := mload(0x0300)
                    let point_2 := mload(0x0340)
                    let point_3 := mload(0x0360)
                    let coeff
                    coeff := addmod(point_0, sub(r, point_2), r)
                    coeff := mulmod(
                        coeff,
                        addmod(point_0, sub(r, point_3), r),
                        r
                    )
                    coeff := mulmod(coeff, mload(0x0380), r)
                    mstore(0x80, coeff)
                    coeff := addmod(point_2, sub(r, point_0), r)
                    coeff := mulmod(
                        coeff,
                        addmod(point_2, sub(r, point_3), r),
                        r
                    )
                    coeff := mulmod(coeff, mload(0x03c0), r)
                    mstore(0xa0, coeff)
                    coeff := addmod(point_3, sub(r, point_0), r)
                    coeff := mulmod(
                        coeff,
                        addmod(point_3, sub(r, point_2), r),
                        r
                    )
                    coeff := mulmod(coeff, mload(0x03e0), r)
                    mstore(0xc0, coeff)
                }
                {
                    let point_1 := mload(0x0320)
                    let point_2 := mload(0x0340)
                    let coeff
                    coeff := addmod(point_1, sub(r, point_2), r)
                    coeff := mulmod(coeff, mload(0x03a0), r)
                    mstore(0xe0, coeff)
                    coeff := addmod(point_2, sub(r, point_1), r)
                    coeff := mulmod(coeff, mload(0x03c0), r)
                    mstore(0x0100, coeff)
                }
                {
                    success := batch_invert(success, 0, 0x0120, r)
                    let diff_0_inv := mload(0x00)
                    mstore(0x0420, diff_0_inv)
                    for {
                        let mptr := 0x0440
                        let mptr_end := 0x04a0
                    } lt(mptr, mptr_end) {
                        mptr := add(mptr, 0x20)
                    } {
                        mstore(mptr, mulmod(mload(mptr), diff_0_inv, r))
                    }
                }
                {
                    let zeta := mload(ZETA_MPTR)
                    let r_eval := 0
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1ae4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1b04), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1a44), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1a64), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x19a4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x19c4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1904), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1924), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1864), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1884), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x17c4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x17e4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1724), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1744), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1684), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x16a4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x15e4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1604), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x15a4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x15c4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1124), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x11c4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x1104), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x11a4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x10e4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1184), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x10c4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1164), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x10a4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1144), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x0ec4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1064), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x0ea4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1084), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x0e84), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1044), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x0e64), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1024), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x20), calldataload(0x0e44), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x40), calldataload(0x1004), r),
                        r
                    )
                    mstore(0x04a0, r_eval)
                }
                {
                    let coeff := mload(0x60)
                    let zeta := mload(ZETA_MPTR)
                    let r_eval := 0
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1344), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, mload(QUOTIENT_EVAL_MPTR), r),
                        r
                    )
                    for {
                        let mptr := 0x14c4
                        let mptr_end := 0x1344
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x20)
                    } {
                        r_eval := addmod(
                            mulmod(r_eval, zeta, r),
                            mulmod(coeff, calldataload(mptr), r),
                            r
                        )
                    }
                    for {
                        let mptr := 0x1324
                        let mptr_end := 0x11c4
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x20)
                    } {
                        r_eval := addmod(
                            mulmod(r_eval, zeta, r),
                            mulmod(coeff, calldataload(mptr), r),
                            r
                        )
                    }
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1b64), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1ac4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1a24), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1984), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x18e4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1844), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x17a4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1704), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(coeff, calldataload(0x1664), r),
                        r
                    )
                    for {
                        let mptr := 0x0fe4
                        let mptr_end := 0x0ec4
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x20)
                    } {
                        r_eval := addmod(
                            mulmod(r_eval, zeta, r),
                            mulmod(coeff, calldataload(mptr), r),
                            r
                        )
                    }
                    r_eval := mulmod(r_eval, mload(0x0440), r)
                    mstore(0x04c0, r_eval)
                }
                {
                    let zeta := mload(ZETA_MPTR)
                    let r_eval := 0
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x80), calldataload(0x1584), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xa0), calldataload(0x1544), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xc0), calldataload(0x1564), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x80), calldataload(0x1524), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xa0), calldataload(0x14e4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xc0), calldataload(0x1504), r),
                        r
                    )
                    r_eval := mulmod(r_eval, mload(0x0460), r)
                    mstore(0x04e0, r_eval)
                }
                {
                    let zeta := mload(ZETA_MPTR)
                    let r_eval := 0
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1b44), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1b24), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1aa4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1a84), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1a04), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x19e4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1964), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1944), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x18c4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x18a4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1824), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1804), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1784), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1764), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x16e4), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x16c4), r),
                        r
                    )
                    r_eval := mulmod(r_eval, zeta, r)
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0xe0), calldataload(0x1644), r),
                        r
                    )
                    r_eval := addmod(
                        r_eval,
                        mulmod(mload(0x0100), calldataload(0x1624), r),
                        r
                    )
                    r_eval := mulmod(r_eval, mload(0x0480), r)
                    mstore(0x0500, r_eval)
                }
                {
                    let sum := mload(0x20)
                    sum := addmod(sum, mload(0x40), r)
                    mstore(0x0520, sum)
                }
                {
                    let sum := mload(0x60)
                    mstore(0x0540, sum)
                }
                {
                    let sum := mload(0x80)
                    sum := addmod(sum, mload(0xa0), r)
                    sum := addmod(sum, mload(0xc0), r)
                    mstore(0x0560, sum)
                }
                {
                    let sum := mload(0xe0)
                    sum := addmod(sum, mload(0x0100), r)
                    mstore(0x0580, sum)
                }
                {
                    for {
                        let mptr := 0x00
                        let mptr_end := 0x80
                        let sum_mptr := 0x0520
                    } lt(mptr, mptr_end) {
                        mptr := add(mptr, 0x20)
                        sum_mptr := add(sum_mptr, 0x20)
                    } {
                        mstore(mptr, mload(sum_mptr))
                    }
                    success := batch_invert(success, 0, 0x80, r)
                    let r_eval := mulmod(mload(0x60), mload(0x0500), r)
                    for {
                        let sum_inv_mptr := 0x40
                        let sum_inv_mptr_end := 0x80
                        let r_eval_mptr := 0x04e0
                    } lt(sum_inv_mptr, sum_inv_mptr_end) {
                        sum_inv_mptr := sub(sum_inv_mptr, 0x20)
                        r_eval_mptr := sub(r_eval_mptr, 0x20)
                    } {
                        r_eval := mulmod(r_eval, mload(NU_MPTR), r)
                        r_eval := addmod(
                            r_eval,
                            mulmod(mload(sum_inv_mptr), mload(r_eval_mptr), r),
                            r
                        )
                    }
                    mstore(R_EVAL_MPTR, r_eval)
                }
                {
                    let nu := mload(NU_MPTR)
                    mstore(0x00, calldataload(0x0c84))
                    mstore(0x20, calldataload(0x0ca4))
                    for {
                        let mptr := 0x0c44
                        let mptr_end := 0x0a04
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x40)
                    } {
                        success := ec_mul_acc(success, mload(ZETA_MPTR))
                        success := ec_add_acc(
                            success,
                            calldataload(mptr),
                            calldataload(add(mptr, 0x20))
                        )
                    }
                    for {
                        let mptr := 0x0504
                        let mptr_end := 0x03c4
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x40)
                    } {
                        success := ec_mul_acc(success, mload(ZETA_MPTR))
                        success := ec_add_acc(
                            success,
                            calldataload(mptr),
                            calldataload(add(mptr, 0x20))
                        )
                    }
                    success := ec_mul_acc(success, mload(ZETA_MPTR))
                    success := ec_add_acc(
                        success,
                        calldataload(0x0144),
                        calldataload(0x0164)
                    )
                    success := ec_mul_acc(success, mload(ZETA_MPTR))
                    success := ec_add_acc(
                        success,
                        calldataload(0x0184),
                        calldataload(0x01a4)
                    )
                    for {
                        let mptr := 0x0104
                        let mptr_end := 0x44
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x40)
                    } {
                        success := ec_mul_acc(success, mload(ZETA_MPTR))
                        success := ec_add_acc(
                            success,
                            calldataload(mptr),
                            calldataload(add(mptr, 0x20))
                        )
                    }
                    mstore(0x80, calldataload(0x0cc4))
                    mstore(0xa0, calldataload(0x0ce4))
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        mload(QUOTIENT_X_MPTR),
                        mload(QUOTIENT_Y_MPTR)
                    )
                    for {
                        let mptr := 0x0dc0
                        let mptr_end := 0x0800
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x40)
                    } {
                        success := ec_mul_tmp(success, mload(ZETA_MPTR))
                        success := ec_add_tmp(
                            success,
                            mload(mptr),
                            mload(add(mptr, 0x20))
                        )
                    }
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0984),
                        calldataload(0x09a4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0904),
                        calldataload(0x0924)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0884),
                        calldataload(0x08a4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0804),
                        calldataload(0x0824)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0784),
                        calldataload(0x07a4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0704),
                        calldataload(0x0724)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0684),
                        calldataload(0x06a4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0604),
                        calldataload(0x0624)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0584),
                        calldataload(0x05a4)
                    )
                    for {
                        let mptr := 0x03c4
                        let mptr_end := 0x0184
                    } lt(mptr_end, mptr) {
                        mptr := sub(mptr, 0x40)
                    } {
                        success := ec_mul_tmp(success, mload(ZETA_MPTR))
                        success := ec_add_tmp(
                            success,
                            calldataload(mptr),
                            calldataload(add(mptr, 0x20))
                        )
                    }
                    success := ec_mul_tmp(success, mulmod(nu, mload(0x0440), r))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    nu := mulmod(nu, mload(NU_MPTR), r)
                    mstore(0x80, calldataload(0x0a04))
                    mstore(0xa0, calldataload(0x0a24))
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x09c4),
                        calldataload(0x09e4)
                    )
                    success := ec_mul_tmp(success, mulmod(nu, mload(0x0460), r))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    nu := mulmod(nu, mload(NU_MPTR), r)
                    mstore(0x80, calldataload(0x0944))
                    mstore(0xa0, calldataload(0x0964))
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x08c4),
                        calldataload(0x08e4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0844),
                        calldataload(0x0864)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x07c4),
                        calldataload(0x07e4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0744),
                        calldataload(0x0764)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x06c4),
                        calldataload(0x06e4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0644),
                        calldataload(0x0664)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x05c4),
                        calldataload(0x05e4)
                    )
                    success := ec_mul_tmp(success, mload(ZETA_MPTR))
                    success := ec_add_tmp(
                        success,
                        calldataload(0x0544),
                        calldataload(0x0564)
                    )
                    success := ec_mul_tmp(success, mulmod(nu, mload(0x0480), r))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    mstore(0x80, mload(G1_X_MPTR))
                    mstore(0xa0, mload(G1_Y_MPTR))
                    success := ec_mul_tmp(success, sub(r, mload(R_EVAL_MPTR)))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    mstore(0x80, calldataload(0x1b84))
                    mstore(0xa0, calldataload(0x1ba4))
                    success := ec_mul_tmp(success, sub(r, mload(0x0400)))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    mstore(0x80, calldataload(0x1bc4))
                    mstore(0xa0, calldataload(0x1be4))
                    success := ec_mul_tmp(success, mload(MU_MPTR))
                    success := ec_add_acc(success, mload(0x80), mload(0xa0))
                    mstore(PAIRING_LHS_X_MPTR, mload(0x00))
                    mstore(PAIRING_LHS_Y_MPTR, mload(0x20))
                    mstore(PAIRING_RHS_X_MPTR, calldataload(0x1bc4))
                    mstore(PAIRING_RHS_Y_MPTR, calldataload(0x1be4))
                }
            }

            // Random linear combine with accumulator
            if mload(HAS_ACCUMULATOR_MPTR) {
                mstore(0x00, mload(ACC_LHS_X_MPTR))
                mstore(0x20, mload(ACC_LHS_Y_MPTR))
                mstore(0x40, mload(ACC_RHS_X_MPTR))
                mstore(0x60, mload(ACC_RHS_Y_MPTR))
                mstore(0x80, mload(PAIRING_LHS_X_MPTR))
                mstore(0xa0, mload(PAIRING_LHS_Y_MPTR))
                mstore(0xc0, mload(PAIRING_RHS_X_MPTR))
                mstore(0xe0, mload(PAIRING_RHS_Y_MPTR))
                let challenge := mod(keccak256(0x00, 0x100), r)

                // [pairing_lhs] += challenge * [acc_lhs]
                success := ec_mul_acc(success, challenge)
                success := ec_add_acc(
                    success,
                    mload(PAIRING_LHS_X_MPTR),
                    mload(PAIRING_LHS_Y_MPTR)
                )
                mstore(PAIRING_LHS_X_MPTR, mload(0x00))
                mstore(PAIRING_LHS_Y_MPTR, mload(0x20))

                // [pairing_rhs] += challenge * [acc_rhs]
                mstore(0x00, mload(ACC_RHS_X_MPTR))
                mstore(0x20, mload(ACC_RHS_Y_MPTR))
                success := ec_mul_acc(success, challenge)
                success := ec_add_acc(
                    success,
                    mload(PAIRING_RHS_X_MPTR),
                    mload(PAIRING_RHS_Y_MPTR)
                )
                mstore(PAIRING_RHS_X_MPTR, mload(0x00))
                mstore(PAIRING_RHS_Y_MPTR, mload(0x20))
            }

            // Perform pairing
            success := ec_pairing(
                success,
                mload(PAIRING_LHS_X_MPTR),
                mload(PAIRING_LHS_Y_MPTR),
                mload(PAIRING_RHS_X_MPTR),
                mload(PAIRING_RHS_Y_MPTR)
            )

            // Revert if anything fails
            if iszero(success) {
                revert(0x00, 0x00)
            }

            // Return 1 as result if everything succeeds
            mstore(0x00, 1)
            return(0x00, 0x20)
        }
    }
}
