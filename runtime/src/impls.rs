use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};

/// Balance of an account.
pub type Balance = u128;

/// IBalance is the signed version of the Balance for orml tokens
pub type IBalance = i128;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

pub type ForeignAssetId = u32;

/// AssetRegistry's AssetProcessor
pub mod asset_registry {
	use frame_support::{
		dispatch::RawOrigin,
		sp_std::marker::PhantomData,
		traits::{EnsureOrigin, EnsureOriginWithArg},
	};
	use orml_traits::asset_registry::{AssetMetadata, AssetProcessor};
	use sp_runtime::DispatchError;

	use super::*;

	#[derive(
		Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
	)]
	pub struct CustomAssetProcessor;

	impl AssetProcessor<CurrencyId, AssetMetadata<Balance, CustomMetadata>> for CustomAssetProcessor {
		fn pre_register(
			id: Option<CurrencyId>,
			metadata: AssetMetadata<Balance, CustomMetadata>,
		) -> Result<(CurrencyId, AssetMetadata<Balance, CustomMetadata>), DispatchError> {
			match id {
				Some(id) => Ok((id, metadata)),
				None => Err(DispatchError::Other("asset-registry: AssetId is required")),
			}
		}

		fn post_register(
			_id: CurrencyId,
			_asset_metadata: AssetMetadata<Balance, CustomMetadata>,
		) -> Result<(), DispatchError> {
			Ok(())
		}
	}

	/// The OrmlAssetRegistry::AuthorityOrigin impl
	pub struct AuthorityOrigin<
		// The origin type
		Origin,
		// The default EnsureOrigin impl used to authorize all
		// assets besides tranche tokens.
		DefaultEnsureOrigin,
	>(PhantomData<(Origin, DefaultEnsureOrigin)>);

	impl<
			Origin: Into<Result<RawOrigin<AccountId>, Origin>> + From<RawOrigin<AccountId>>,
			DefaultEnsureOrigin: EnsureOrigin<Origin>,
		> EnsureOriginWithArg<Origin, Option<CurrencyId>> for AuthorityOrigin<Origin, DefaultEnsureOrigin>
	{
		type Success = ();

		fn try_origin(
			origin: Origin,
			asset_id: &Option<CurrencyId>,
		) -> Result<Self::Success, Origin> {
			match asset_id {
				// Any other `asset_id` defaults to EnsureRoot
				_ => DefaultEnsureOrigin::try_origin(origin).map(|_| ()),
			}
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin(_asset_id: &Option<CurrencyId>) -> Origin {
			unimplemented!()
		}
	}
}

/// A type describing our custom additional metadata stored in the OrmlAssetRegistry.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Clone,
	Copy,
	Default,
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct CustomMetadata {
	/// XCM-related metadata.
	pub xcm: super::XcmMetadata,

	/// Whether an asset can be minted.
	/// When `true`, the right permissions will checked in the permissions
	/// pallet to authorize asset minting by an origin.
	pub mintable: bool,

	/// Whether an asset is _permissioned_, i.e., whether the asset can only
	/// be transferred from and to whitelisted accounts. When `true`, the
	/// right permissions will checked in the permissions pallet to authorize
	/// transfer between mutually allowed from and to accounts.
	pub permissioned: bool,

	/// Whether an asset can be used as a currency to fund Centrifuge Pools.
	pub pool_currency: bool,
}

#[derive(
	Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	/// A foreign asset
	ForeignAsset(ForeignAssetId),
}

impl Default for CurrencyId {
	fn default() -> Self {
		CurrencyId::ForeignAsset(0)
	}
}

// A way to generate different currencies from a number.
// Can be used in tests/benchmarks to generate different currencies.
impl From<u32> for CurrencyId {
	fn from(value: u32) -> Self {
		CurrencyId::ForeignAsset(value)
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(
	Clone,
	Copy,
	Default,
	PartialOrd,
	Ord,
	PartialEq,
	Eq,
	Debug,
	Encode,
	Decode,
	TypeInfo,
	MaxEncodedLen,
)]
pub struct XcmMetadata {
	/// The fee charged for every second that an XCM message takes to execute.
	/// When `None`, the `default_per_second` will be used instead.
	pub fee_per_second: Option<Balance>,
}
