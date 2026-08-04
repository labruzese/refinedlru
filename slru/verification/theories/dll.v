From radium Require Import lang notation.
From refinedrust Require Import typing shims.

Section slru_dll.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.

  (* Ownership of a single node. *)
  Context (node_own :
    thread_id → loc →
    option (place_rfn K_rt) → option (place_rfn V_rt) →
    loc →                        (* prev *)
    loc →                        (* next *)
    iProp Σ).

  Implicit Types (hd tl : loc) (locs : list loc) (l : list (RT_xt K_rt * RT_xt V_rt)).

  (* 
  Index arithmetic
      A cache holding [l = [x0; ..; x_{n-1}]] is laid out as
        hd  <->  m0  <->  ..  <->  m_{n-1}  <->  tl
      with [hd.prev = NULL] and [tl.next = NULL]. 
      [dll_chain] is the full sequence including both sentinels;
      the key/val/prev/next helpers below are all indexed by position
       *in that chain*, so every [∗ list] index lines up with no bookkeeping. 
  *)

  Definition dll_chain hd locs tl : list loc := hd :: locs ++ [tl].

  Definition dll_keys l : list (option (place_rfn K_rt)) :=
    None :: ((λ kv, Some (# ($# kv.1))) <$> l) ++ [None].

  Definition dll_vals l : list (option (place_rfn V_rt)) :=
    None :: ((λ kv, Some (# ($# kv.2))) <$> l) ++ [None].

  (* Successor in the chain; NULL for the tail sentinel. *)
  Definition dll_next (chain : list loc) (i : nat) : loc :=
    default NULL_loc (chain !! S i).

  (* Predecessor in the chain; NULL for the head sentinel. Prepending one
     NULL_loc avoids the truncated-subtraction trap at i = 0. *)
  Definition dll_prev (chain : list loc) (i : nat) : loc :=
    default NULL_loc ((NULL_loc :: chain) !! i).

  Definition slru_dll (π : thread_id) hd tl l : iProp Σ :=
    ∃ locs : list loc,
      ⌜length locs = length l⌝ ∗
      ⌜Forall (λ ln, ln.(loc_a) ≠ 0) (dll_chain hd locs tl)⌝ ∗
      ([∗ list] i ↦ ln ∈ dll_chain hd locs tl,
         node_own π ln
           (dll_keys l !!! i) (dll_vals l !!! i)
           (dll_prev (dll_chain hd locs tl) i)
           (dll_next (dll_chain hd locs tl) i)).
End slru_dll.

(* Let resolution and Lithium see through these.*)
Global Typeclasses Transparent slru_dll dll_chain dll_keys dll_vals dll_next dll_prev.
Global Hint Unfold slru_dll dll_chain dll_keys dll_vals dll_next dll_prev : tyunfold.

Section lemmas.
  Context `{RRGS : !refinedrustGS Σ}.
  Context {K_rt V_rt : RT}.
  Context (node_own :
    thread_id → loc →
    option (place_rfn K_rt) → option (place_rfn V_rt) →
    loc → loc → iProp Σ).

  Implicit Types (hd tl : loc) (l : list (RT_xt K_rt * RT_xt V_rt)).

  (* for [LruCache::new] *)

  Lemma slru_dll_nil π hd tl :
    hd.(loc_a) ≠ 0 → tl.(loc_a) ≠ 0 →
    node_own π hd None None NULL_loc tl ∗
    node_own π tl None None hd NULL_loc
    ⊢ slru_dll node_own π hd tl [].
  Proof.
    iIntros (Hhd Htl) "(Hh & Ht)".
    iExists []. rewrite /dll_chain /dll_keys /dll_vals /=.
    iSplitR; first done.
    iSplitR.
    { iPureIntro. by repeat constructor. }
    rewrite /dll_next /dll_prev /=. iFrame.
  Qed.

  (** Shape facts the code needs *)
  Lemma dll_chain_length hd locs tl :
    length (dll_chain hd locs tl) = S (S (length locs)).
  Proof. rewrite /dll_chain /= length_app /=. lia. Qed.

  (* [find] starts at [self.head.next]: the first payload node, or [tl]. *)
  Lemma dll_next_head hd locs tl :
    dll_next (dll_chain hd locs tl) 0 = default tl (head locs).
  Proof. destruct locs; done. Qed.

  (* The eviction victim in the third branch of [put] is [tl.prev]. *)
  Lemma dll_prev_tail hd locs tl :
    dll_prev (dll_chain hd locs tl) (S (length locs)) = default hd (last locs).
  Proof.
    rewrite /dll_prev /dll_chain.
    change ((NULL_loc :: hd :: locs ++ [tl]) !! S (length locs))
      with (((hd :: locs) ++ [tl]) !! length locs).
    rewrite lookup_app_l; last (simpl; lia).
    replace (length locs) with (pred (length (hd :: locs))) by (simpl; lia).
    rewrite -last_lookup last_cons. by destruct (last locs).
  Qed.

  (** ** Surgery, one lemma per unsafe operation in the Rust code

      These are the ones that were impossible to state usefully against the
      Fixpoint version. Each is now a [big_sepL] rearrangement: open at an
      index, hand out the node, and close with the neighbours repointed.
      Proofs left open -- fiddly but not deep;
      [big_sepL_insert_acc] / [big_sepL_app] do most of the work. *)

  (* Open the [∗ list] at payload index [i], keeping the frame. This is the
     workhorse: `get`, both `find` hits, and the eviction branch are all
     "open at i, mutate, close". *)
  Lemma slru_dll_acc π hd tl l i x :
    l !! i = Some x →
    slru_dll node_own π hd tl l -∗
    ∃ ln prev next,
      node_own π ln (Some (# ($# x.1))) (Some (# ($# x.2))) prev next ∗
      (∀ x', node_own π ln (Some (# ($# x'.1))) (Some (# ($# x'.2))) prev next -∗
             slru_dll node_own π hd tl (<[i := x']> l)).
  Proof. Admitted.

  (* `detach` at payload index [i], after the caller has patched
     [prev.next] and [next.prev]. *)
  Lemma slru_dll_detach π hd tl l i x :
    l !! i = Some x →
    slru_dll node_own π hd tl l -∗
    ∃ ln prev next,
      node_own π ln (Some (# ($# x.1))) (Some (# ($# x.2))) prev next ∗
      slru_dll node_own π hd tl (delete i l).
  Proof. Admitted.

  (* `head.attach(node)`: splice in at the MRU end. *)
  Lemma slru_dll_attach_front π hd tl l ln x prev next :
    ln.(loc_a) ≠ 0 →
    node_own π ln (Some (# ($# x.1))) (Some (# ($# x.2))) prev next -∗
    slru_dll node_own π hd tl l -∗
    slru_dll node_own π hd tl (x :: l).
  Proof. Admitted.

  (* detach-then-attach, i.e. the "touch" performed by `get` and by the
     `Some` branch of `put`. No allocation: the result is a rotation. *)
  Lemma slru_dll_touch π hd tl l i x :
    l !! i = Some x →
    slru_dll node_own π hd tl l -∗
    slru_dll node_own π hd tl (x :: delete i l).
  Proof. Admitted.

  (* The eviction branch of `put`: reuse the LRU node's allocation. *)
  Lemma slru_dll_evict_reuse π hd tl l x y :
    last l = Some y →
    slru_dll node_own π hd tl l -∗
    slru_dll node_own π hd tl (x :: removelast l).
  Proof. Admitted.

  (* `Drop`: peel one node off the front of the chain. *)
  Lemma slru_dll_uncons π hd tl l :
    slru_dll node_own π hd tl l -∗
    ∃ next, node_own π hd None None NULL_loc next ∗
      ((⌜l = [] ∧ next = tl⌝ ∗ node_own π tl None None hd NULL_loc) ∨
       (∃ x l', ⌜l = x :: l'⌝ ∗ slru_dll node_own π next tl l')).
  Proof. Admitted.

End lemmas.