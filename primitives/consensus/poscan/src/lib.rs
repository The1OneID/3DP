// This file is part of 3DPass.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// Copyright (C) 2022 3DPass
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Primitives for Substrate Proof-of-Scan (PoScan) consensus.

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;
use sp_runtime::ConsensusEngineId;
use codec::Decode;
use lzss::{Lzss, SliceReader, VecWriter};

/// The `ConsensusEngineId` of PoScan.
pub const POSCAN_ENGINE_ID: ConsensusEngineId = [b'p', b'o', b's', b'c'];
pub const POSCAN_COIN_ID: u8 = 72;

pub const MAX_MINING_OBJ_LEN: usize = 100 * 1024;

/// Type of seal.
pub type Seal = Vec<u8>;
pub type Difficulty = sp_core::U256;

/// Block interval, in seconds, the network will tune its next_target for.
pub const BLOCK_TIME_SEC: u64 = 60;
/// Block time interval in milliseconds.
pub const BLOCK_TIME: u64 = BLOCK_TIME_SEC * 1000;

/// Nominal height for standard time intervals, hour is 60 blocks
pub const HOUR_HEIGHT: u64 = 3600 / BLOCK_TIME_SEC;
/// A day is 1440 blocks
pub const DAY_HEIGHT: u64 = 24 * HOUR_HEIGHT;
/// A week is 10_080 blocks
pub const WEEK_HEIGHT: u64 = 7 * DAY_HEIGHT;
/// A year is 524_160 blocks
pub const YEAR_HEIGHT: u64 = 52 * WEEK_HEIGHT;

/// Number of blocks used to calculate difficulty adjustments
pub const DIFFICULTY_ADJUST_WINDOW: u64 = HOUR_HEIGHT;
/// Clamp factor to use for difficulty adjustment
/// Limit value to within this factor of goal
pub const CLAMP_FACTOR: u128 = 2;
/// Dampening factor to use for difficulty adjustment
pub const DIFFICULTY_DAMP_FACTOR: u128 = 3;
/// Minimum difficulty, enforced in diff retargetting
/// avoids getting stuck when trying to increase difficulty subject to dampening
pub const MIN_DIFFICULTY: u128 = DIFFICULTY_DAMP_FACTOR;
/// Maximum difficulty.
pub const MAX_DIFFICULTY: u128 = u128::max_value();

/// Value of 1 KLP.
pub const DOLLARS: u128 = 1_000_000_000_000;
/// Value of cents relative to 3DP.
pub const CENTS: u128 = DOLLARS / 100;
/// Value of millicents relative to 3DP.
pub const MILLICENTS: u128 = CENTS / 1_000;
/// Value of microcents relative to 3DP.
pub const MICROCENTS: u128 = MILLICENTS / 1_000;

pub const fn deposit(items: u32, bytes: u32) -> u128 {
	items as u128 * 2 * DOLLARS + (bytes as u128) * 10 * MILLICENTS
}

/// Block number of one hour.
pub const HOURS: u32 = 60;
/// Block number of one day.
pub const DAYS: u32 = 24 * HOURS;


/// Define methods that total difficulty should implement.
pub trait TotalDifficulty {
	fn increment(&mut self, other: Self);
}

impl TotalDifficulty for sp_core::U256 {
	fn increment(&mut self, other: Self) {
		let ret = self.saturating_add(other);
		*self = ret;
	}
}

impl TotalDifficulty for u128 {
	fn increment(&mut self, other: Self) {
		let ret = self.saturating_add(other);
		*self = ret;
	}
}

sp_api::decl_runtime_apis! {
	/// API necessary for timestamp-based difficulty adjustment algorithms.
	pub trait TimestampApi<Moment: Decode> {
		/// Return the timestamp in the current block.
		fn timestamp() -> Moment;
	}

	/// API for those chains that put their difficulty adjustment algorithm directly
	/// onto runtime. Note that while putting difficulty adjustment algorithm to
	/// runtime is safe, putting the PoW algorithm on runtime is not.
	pub trait DifficultyApi<Difficulty: Decode> {
		/// Return the target difficulty of the next block.
		fn difficulty() -> Difficulty;
	}

	pub trait AlgorithmApi {
		fn identifier() -> [u8; 8];
	}
}

pub fn compress_obj(obj: &[u8]) -> Vec<u8> {
	type MyLzss = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;
	let result = MyLzss::compress(
		SliceReader::new(obj),
		VecWriter::with_capacity(4096),
	);

	result.unwrap()
}

pub fn decompress_obj(obj: &[u8]) -> Vec<u8> {
	type MyLzss = Lzss<10, 4, 0x20, { 1 << 10 }, { 2 << 10 }>;
	let result = MyLzss::decompress(
		SliceReader::new(obj),
		VecWriter::with_capacity(4096),
	);

	result.unwrap()
}
