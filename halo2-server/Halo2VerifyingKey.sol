// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

contract Halo2VerifyingKey {
    constructor() {
        assembly {
            mstore(0x0000, 0x2a40a040072899e1a43cc081672b16ae211b1c2c564ea43b0980030dcb145305) // vk_digest
            mstore(0x0020, 0x0000000000000000000000000000000000000000000000000000000000000004) // num_instances
            mstore(0x0040, 0x0000000000000000000000000000000000000000000000000000000000000014) // k
            mstore(0x0060, 0x30644b6c9c4a72169e4daa317d25f04512ae15c53b34e8f5acd8e155d0a6c101) // n_inv
            mstore(0x0080, 0x2a14464f1ff42de3856402b62520e670745e39fada049d5b2f0e1e3182673378) // omega
            mstore(0x00a0, 0x220db0d8bf832baf9eecbf4fa49947e0b2a3d31df0a733ea5ae8abbdab442d5f) // omega_inv
            mstore(0x00c0, 0x1d70265fc2e33776f0609a8b65dc22789d415875873f53a4a63e7d88ff30707f) // omega_inv_to_l
            mstore(0x00e0, 0x0000000000000000000000000000000000000000000000000000000000000000) // has_accumulator
            mstore(0x0100, 0x0000000000000000000000000000000000000000000000000000000000000000) // acc_offset
            mstore(0x0120, 0x0000000000000000000000000000000000000000000000000000000000000000) // num_acc_limbs
            mstore(0x0140, 0x0000000000000000000000000000000000000000000000000000000000000000) // num_acc_limb_bits
            mstore(0x0160, 0x0000000000000000000000000000000000000000000000000000000000000001) // g1_x
            mstore(0x0180, 0x0000000000000000000000000000000000000000000000000000000000000002) // g1_y
            mstore(0x01a0, 0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2) // g2_x_1
            mstore(0x01c0, 0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed) // g2_x_2
            mstore(0x01e0, 0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b) // g2_y_1
            mstore(0x0200, 0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa) // g2_y_2
            mstore(0x0220, 0x22b272d1cbd20fd069ce9bb3b460b46465181e70ef7a282f1385a78511d2db6b) // neg_s_g2_x_1
            mstore(0x0240, 0x003496779710f8144162767d0556d4463ddb67e038321b4e99582fc9409558c3) // neg_s_g2_x_2
            mstore(0x0260, 0x3036e6ff58fe9b29fd3ecbcfb83474fa949417689d76a6efec1e46d5b287637c) // neg_s_g2_y_1
            mstore(0x0280, 0x02eb5fc7345c81b07f82c7f40cfcc772f8c946eec1e49781e87231eb93b41781) // neg_s_g2_y_2
            mstore(0x02a0, 0x05fef3180e0315768f989fbf0694ef56704a48038241f4a1b196af0fdef3c1b7) // fixed_comms[0].x
            mstore(0x02c0, 0x28756b2f7ed68f7391d5bbd007e091917110d6e3d7e93d8bcd88365e78f59f37) // fixed_comms[0].y
            mstore(0x02e0, 0x190859ae52fdf3627a04f001f7943139db54eb527342e609c9031002ee6a43a3) // fixed_comms[1].x
            mstore(0x0300, 0x1c20235184b62c358a18305343196878fd5197a34c7b02a62aced7e0e67d2d3e) // fixed_comms[1].y
            mstore(0x0320, 0x124c25180843eb5f934f08803c633119bd7d0061b3c72461f054e21e08a48554) // fixed_comms[2].x
            mstore(0x0340, 0x14fddceec100338d76aedae280716a44f47bf0970579400068b76aae139c4ff6) // fixed_comms[2].y
            mstore(0x0360, 0x081553d0e0ec182919e987c91677449af584608a33adb89cfc335cd03b4b5d88) // fixed_comms[3].x
            mstore(0x0380, 0x027bdcc0e8831c8a729a7c29cbab6425aca76932141c26f31351a2f8861c0e18) // fixed_comms[3].y
            mstore(0x03a0, 0x0e63b187734a504416730ad825e57f5104ac5ea47d7a501d3e558deefe8f4dda) // fixed_comms[4].x
            mstore(0x03c0, 0x1331848e3861daad5b024a86a5c6feb2842b1902a9fe107a1d1e270ecb8e02b3) // fixed_comms[4].y
            mstore(0x03e0, 0x28820206e1c40475f60f4ac0be221348bb8a6db2d4a2b22abd2a3c72d6007e6b) // fixed_comms[5].x
            mstore(0x0400, 0x082222283bab8ff87ba6255659fd172465697002e696f9117af938cff68b085e) // fixed_comms[5].y
            mstore(0x0420, 0x09d1ada5201dfbc210f55193679ed0788e05aa69fd3cf50cf790f8bfacc97661) // fixed_comms[6].x
            mstore(0x0440, 0x2dcfb54be705d73043663e83270feb0d445b89bdcf93a2d9557eb8bf8cb81a82) // fixed_comms[6].y
            mstore(0x0460, 0x0e06878b17d5879bf4ad8459b4a446760df9dcac442645c69bbdc32832f5d53e) // fixed_comms[7].x
            mstore(0x0480, 0x2bba2bc73c577260515939d10dbcd7d0f8f07a6971234ab38e5445d02ad21a56) // fixed_comms[7].y
            mstore(0x04a0, 0x172e8d2158a9fc3dda8a075d52303933f572819ffd47b98e47ae47e1f5f7ef9b) // fixed_comms[8].x
            mstore(0x04c0, 0x043f2cc44bc9f2187baba046fa0d3773daae1402694da656c4f73dd954ada19f) // fixed_comms[8].y
            mstore(0x04e0, 0x20edafae1453b59e676a2be9034522e97c72e79ae05256a7aad0b4f08181792e) // fixed_comms[9].x
            mstore(0x0500, 0x0e295f65ad6690fdb11bf15f9f895d78c92e10ba15426ebdce410f91ffb0336b) // fixed_comms[9].y
            mstore(0x0520, 0x06ce0fb9765d4d716283c062a5b450c3edb2bf66ab25ca1f3fbaa5e6b83d0e89) // fixed_comms[10].x
            mstore(0x0540, 0x1383f7dc7baec9c9614a8a867946a75e7641af0f51ece5e8219c5a28d012fa2a) // fixed_comms[10].y
            mstore(0x0560, 0x2fae963e439c93a68ee29c23e00b0ef1117eb057057f4b52ede2efc7ba0fd01a) // permutation_comms[0].x
            mstore(0x0580, 0x08e5c7244ad2c723fd0a884ab1bce39feb04c4a6e82aff229f4dc3b6065b2e2a) // permutation_comms[0].y
            mstore(0x05a0, 0x0e982bdde49589bdfbd4c4507371363b80c1df73a489f69fdce0b9221291025f) // permutation_comms[1].x
            mstore(0x05c0, 0x08c4e1c5ba97ce85a392e029d8d811540fded9d5ee2b5e9e375cf5a4f4ce224f) // permutation_comms[1].y
            mstore(0x05e0, 0x059c9072edc4b500609b07e66dbca936985caac59413edd413437b652bbfe2cd) // permutation_comms[2].x
            mstore(0x0600, 0x00a781de8aad840f4cc6dce8e9f030c8357761b650779559c30c05e772dc9bea) // permutation_comms[2].y
            mstore(0x0620, 0x127b9aaa8b0e71fe4b55c2e7166eadf16439263bf482834539f6a057de07e52b) // permutation_comms[3].x
            mstore(0x0640, 0x07196d73ac7705dde0473d4d0a47e2a0e65d7d51e95509d7eaf2bf6ef7e85641) // permutation_comms[3].y
            mstore(0x0660, 0x257ad3ad9f2888389ab0ea989b821f15234b5db9c2a446451f535557e011bd47) // permutation_comms[4].x
            mstore(0x0680, 0x227cb8cbb4b7fd83094dce4300e95328098bcd203155475aa4141a784852a6fc) // permutation_comms[4].y
            mstore(0x06a0, 0x2738af849828010db1500728bb600af77e296517fff09358d6fa62308e272d63) // permutation_comms[5].x
            mstore(0x06c0, 0x218c18732e7155f90db68ec18a8b2441952130fc7cafd6335bcbcf8c4cd9b17a) // permutation_comms[5].y
            mstore(0x06e0, 0x153e8677c6b4ba89e772e2b461c52ead83f7ea1eca154be8a4f026944fa48811) // permutation_comms[6].x
            mstore(0x0700, 0x12e581299a651e3f75921a33141d030f27c1acb23f101ef6cac4f7a0fe250985) // permutation_comms[6].y
            mstore(0x0720, 0x21fdf430858073e5c5e9ab9837b9079f5f9863e2f33f1b35602032bce44897bb) // permutation_comms[7].x
            mstore(0x0740, 0x0feba46e8c1550c4b51d8e5601a79991bbc13c332c767cec17df2e49fe6ced0b) // permutation_comms[7].y
            mstore(0x0760, 0x2fffd1bdce1db6f21ced1f9490edf1b0dfb7d9727e32b07356d6da9525320925) // permutation_comms[8].x
            mstore(0x0780, 0x1d0e8a441796ad0671b2e65a3d5e9f6ae0269516f739c149dffc72ace4e2161d) // permutation_comms[8].y
            mstore(0x07a0, 0x29ec78da3f80e28a8648ca4ab7539c6037226ccdf6410448c1aa7edd871c9a70) // permutation_comms[9].x
            mstore(0x07c0, 0x0dd72877744ffe46161aa42a2833402ad6c69a2724664a870e9cba88aba1e984) // permutation_comms[9].y
            mstore(0x07e0, 0x13bdb73d8ddb38373dca7a31ad9be672dbdcfdc68f182459d7ebc373317fb512) // permutation_comms[10].x
            mstore(0x0800, 0x082878013b7cdf041df2c687e308c5fd9d545c150f26ccd9cc5f3692003fda30) // permutation_comms[10].y
            mstore(0x0820, 0x2bfff3f490ee40c2be13f8655cdce22a7f5404e8033c97ffff57b143d0dda3fa) // permutation_comms[11].x
            mstore(0x0840, 0x28e963c97ec0317e27c5b618511525c00dfef30fcd4c821354f44e794b2e08fb) // permutation_comms[11].y

            return(0, 0x0860)
        }
    }
}