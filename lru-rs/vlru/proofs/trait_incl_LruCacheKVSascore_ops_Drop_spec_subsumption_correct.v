From radium Require Import lang notation.
From refinedrust Require Import typing shims.
From vlru.vlru.generated Require Import generated_specs_vlru.

Set Default Proof Using "Type".

Section proof.
Context `{RRGS : !refinedrustGS Σ}.

Lemma LruCacheKVSascore_ops_Drop_spec_subsumption_correct  : (LruCacheKVSascore_ops_Drop_spec_subsumption).
Proof.
  unfold LruCacheKVSascore_ops_Drop_spec_subsumption; solve_trait_incl_prelude.
  all: repeat liRStep; liShow.
  all: print_remaining_trait_goal.
  Unshelve.
  all: sidecond_solver.
  Unshelve.
  all: sidecond_hammer.
Qed.

End proof.
