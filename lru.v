From stdpp Require Import prelude list.

Record lru (K V : Type) := {
  capacity : nat;
  order    : list (K * V);
}.
Arguments capacity {K V}.
Arguments order    {K V}.

Definition well_formed {K V} `{EqDecision K} (cache : lru K V) : Prop :=
  length cache.(order) ≤ cache.(capacity) ∧
  NoDup (cache.(order).*1).

Definition put {K V} `{EqDecision K}
    (key : K) (value : V) (cache : lru K V) : lru K V * option V.

Definition get {K V} `{EqDecision K}
    (key : K) (cache : lru K V) : option V * lru K V.

Definition pop {K V} `{EqDecision K}
    (cache : lru K V) : option (K * V) * lru K V.

Definition peek {K V} `{EqDecision K}
    (key : K) (cache : lru K V) : option V.

(* put on a fresh key becomes MRU *)
Theorem put_fresh_mru {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V) (evicted : option V),
  well_formed cache →
  key ∉ cache.(order).*1 →
  put key value cache = (cache', evicted) →
  cache'.(order) !! 0 = Some (key, value).

(* P2: put on a fresh key at capacity evicts the LRU entry *)
Theorem put_fresh_at_capacity_evicts {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V) (evicted : option V),
  well_formed cache →
  key ∉ cache.(order).*1 →
  length cache.(order) = cache.(capacity) →
  cache.(capacity) > 0 →
  put key value cache = (cache', evicted) →
  evicted = Some (last cache.(order) (key, value)).1 ∧
  length cache'.(order) = cache.(capacity).

(* P3: put on an existing key updates the value and promotes to MRU *)
Theorem put_existing_updates_and_promotes {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V) (evicted : option V),
  well_formed cache →
  key ∈ cache.(order).*1 →
  put key value cache = (cache', evicted) →
  cache'.(order) !! 0 = Some (key, value) ∧
  evicted = None ∧
  length cache'.(order) = length cache.(order).

(* P4: get returns the value if present *)
Theorem get_returns_value {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V),
  well_formed cache →
  (key, value) ∈ cache.(order) →
  fst (get key cache) = Some value.

(* P5: get promotes to MRU *)
Theorem get_promotes {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V),
  well_formed cache →
  (key, value) ∈ cache.(order) →
  get key cache = (Some value, cache') →
  cache'.(order) !! 0 = Some (key, value) ∧
  length cache'.(order) = length cache.(order) ∧
  cache'.(order).*1 ≡ₚ cache.(order).*1.

(* P6: get on absent key returns None and doesn't modify cache *)
Theorem get_absent {K V} `{EqDecision K} :
  ∀ (key : K) (cache cache' : lru K V),
  well_formed cache →
  key ∉ cache.(order).*1 →
  get key cache = (None, cache).

(* P7: well-formedness is preserved by all operations *)
Theorem put_well_formed {K V} `{EqDecision K} :
  ∀ (key : K) (value : V) (cache cache' : lru K V) (evicted : option V),
  well_formed cache →
  put key value cache = (cache', evicted) →
  well_formed cache'.

Theorem get_well_formed {K V} `{EqDecision K} :
  ∀ (key : K) (cache cache' : lru K V) (result : option V),
  well_formed cache →
  get key cache = (result, cache') →
  well_formed cache'.

Theorem pop_well_formed {K V} `{EqDecision K} :
  ∀ (cache cache' : lru K V) (evicted : option (K * V)),
  well_formed cache →
  pop cache = (evicted, cache') →
  well_formed cache'.