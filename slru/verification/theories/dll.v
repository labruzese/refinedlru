(** * A doubly-linked list with sentinels, for the SLRU cache.

    This file is meant to live in [case_studies/extra_proofs/slru/slru_dll.v]
    (or wherever you keep your extra proofs) and be pulled into the crate with

      #![rr::import("refinedrust.extra_proofs.slru", "slru_dll")]

    with a sibling [dune] file (see the bottom of this file for its contents).

    KEY DESIGN POINT
    ================
    [extra_proofs] are compiled *before* RefinedRust's generated code, so this
    file cannot mention [ListNode_ty] / [ListNode_sls] -- those are generated.
    We therefore abstract over the per-node ownership as a parameter [node]:

      node ln kv nx pv

    should assert full ownership of the node allocated at [ln], carrying
    payload [kv] ([None] for the two sentinels), whose [next] field is [nx] and
    whose [prev] field is [pv].

    All the "linked-list-ness" (shape, coherence of prev/next, disjointness of
    node addresses) lives here and is proved once. The Rust-side invariant
    attribute only has to say what a *single* node owns, which is where the
    generated type names are allowed to appear.

    STATUS: definitions are complete; the surgery lemmas at the bottom are
    stated but not yet proved (marked [Admitted]). Nothing here has been
    machine-checked -- expect to fix small notation/universe issues on the
    first [dune build]. *)

From radium Require Import lang notation.
From refinedrust Require Import typing.

Section dll.
  Context `{!refinedrustGS Σ}.
  Context {K V : Type}.

  Implicit Types (node : loc → option (K * V) → loc → loc → iProp Σ).
  Implicit Types (hd tl : loc) (l : list (K * V)) (mid : list loc).

  (** ** The physical chain

      A cache with payload [l = [x0; ...; x_{n-1}]] is physically laid out as

        hd  <->  m0  <->  m1  <->  ...  <->  m_{n-1}  <->  tl

      where [hd] and [tl] are the two sentinel nodes ([key = None], [val = None]),
      [hd.prev = NULL] and [tl.next = NULL].

      Note the order convention: [hd] is the most-recently-used end (that is
      where [attach] inserts), [tl.prev] is the eviction victim. So [l] is
      ordered most-recently-used first. *)

  Definition dll_chain hd mid tl : list loc := hd :: mid ++ [tl].

  (** The payload sitting at each position of [dll_chain]: [None] at the two
      sentinels, [Some x] in between. Indices line up with [dll_chain]. *)
  Definition dll_payloads l : list (option (K * V)) :=
    None :: (Some <$> l) ++ [None].

  (** The [next] field of the node at position [i]: the successor in the chain,
      or [NULL] for the tail sentinel. *)
  Definition dll_next (chain : list loc) (i : nat) : loc :=
    default NULL_loc (chain !! S i).

  (** The [prev] field of the node at position [i]: the predecessor in the
      chain, or [NULL] for the head sentinel. Shifting by one [NULL_loc]
      avoids the [i - 1] truncated-subtraction trap at [i = 0]. *)
  Definition dll_prev (chain : list loc) (i : nat) : loc :=
    default NULL_loc ((NULL_loc :: chain) !! i).

  (** ** The main predicate *)

  Definition slru_dll node hd tl l : iProp Σ :=
    ∃ mid : list loc,
      (* the chain has exactly one interior node per payload element *)
      ⌜length mid = length l⌝ ∗
      (* node addresses are pairwise distinct -- this is what makes the
         [∗ list] below a genuine separating conjunction over *distinct*
         allocations, and it is what [detach] needs in order to know that
         updating a neighbour does not alias the node being removed *)
      ⌜NoDup (dll_chain hd mid tl)⌝ ∗
      (* every node was heap-allocated, so no address is null; needed to turn
         ownership back into a [Box] on drop, and to justify the [!is_null]
         reasoning in the code *)
      ⌜Forall (λ ln, ln.(loc_a) ≠ 0) (dll_chain hd mid tl)⌝ ∗
      ([∗ list] i ↦ ln ∈ dll_chain hd mid tl,
        ∃ kv, ⌜dll_payloads l !! i = Some kv⌝ ∗
          node ln kv
            (dll_next (dll_chain hd mid tl) i)
            (dll_prev (dll_chain hd mid tl) i)).

  Global Instance slru_dll_ne node hd tl l n :
    Proper (pointwise_relation _ (pointwise_relation _
      (pointwise_relation _ (pointwise_relation _ (dist n)))) ==> dist n)
      (λ node, slru_dll node hd tl l).
  Proof. solve_proper. Qed.

  Lemma slru_dll_mono node1 node2 hd tl l :
    (∀ ln kv nx pv, node1 ln kv nx pv -∗ node2 ln kv nx pv) -∗
    slru_dll node1 hd tl l -∗ slru_dll node2 hd tl l.
  Proof.
    iIntros "Hmono (%mid & %Hlen & %Hnodup & %Hnn & Hl)".
    iExists mid. iFrame "%".
    iApply (big_sepL_impl with "Hl").
    iIntros "!>" (i ln _) "(%kv & %Hkv & Hn)".
    iExists kv. iSplitR; first done. by iApply "Hmono".
  Qed.

  (** ** Pure shape facts

      These are the facts the *code* needs: they justify the null checks, the
      [tail.prev] eviction lookup, and the termination of [find]. *)

  Lemma dll_chain_length hd mid tl :
    length (dll_chain hd mid tl) = S (S (length mid)).
  Proof. rewrite /dll_chain /= length_app /=. lia. Qed.

  Lemma dll_payloads_length l :
    length (dll_payloads l) = S (S (length l)).
  Proof. rewrite /dll_payloads /= length_app length_fmap /=. lia. Qed.

  Lemma dll_chain_head hd mid tl :
    dll_chain hd mid tl !! 0%nat = Some hd.
  Proof. done. Qed.

  Lemma dll_chain_last hd mid tl :
    dll_chain hd mid tl !! S (length mid) = Some tl.
  Proof.
    rewrite /dll_chain /= lookup_app_r; last lia.
    by rewrite Nat.sub_diag.
  Qed.

  (** The head sentinel's [next] is the first payload node, or [tl] when empty.
      This is what makes [find] starting at [self.head.next] correct. *)
  Lemma dll_next_head hd mid tl :
    dll_next (dll_chain hd mid tl) 0 = default tl (head mid).
  Proof. destruct mid; done. Qed.

  (** The tail sentinel's [prev] is the least-recently-used node -- the
      eviction victim in the [None] branch of [put]. *)
  Lemma dll_prev_tail hd mid tl :
    dll_prev (dll_chain hd mid tl) (S (length mid)) = default hd (last mid).
  Proof. (* by [rev_ind] on [mid]; the [mid = mid' ++ [x]] case is
            [last_snoc] + [lookup_app_r] *) Admitted.

  (** Sentinels are never null, so [!cur.is_null()] in [Drop] terminates
      exactly when it walks off the end of the chain. *)
  Lemma slru_dll_hd_tl_ne node hd tl l :
    slru_dll node hd tl l -∗ ⌜hd ≠ tl⌝.
  Proof.
    iIntros "(%mid & _ & %Hnodup & _ & _)".
    iPureIntro. rewrite /dll_chain in Hnodup.
    apply NoDup_cons in Hnodup as [Hnotin _].
    intros ->. apply Hnotin, elem_of_app. right. by apply elem_of_list_singleton.
  Qed.

  Lemma slru_dll_not_null node hd tl l :
    slru_dll node hd tl l -∗ ⌜hd.(loc_a) ≠ 0 ∧ tl.(loc_a) ≠ 0⌝.
  Proof.
    iIntros "(%mid & _ & _ & %Hnn & _)".
    iPureIntro. rewrite /dll_chain in Hnn.
    apply Forall_cons in Hnn as [? Hnn].
    apply Forall_app in Hnn as [_ Hnn].
    apply Forall_singleton in Hnn. done.
  Qed.

  (** ** Access lemmas

      [slru_dll_acc] is the workhorse: it hands out ownership of one node
      together with a frame that gives the list back. Every method below is an
      instance of "open at some index, mutate, close". *)

  (** NOTE. The general accessor you might reach for first is:

      Lemma slru_dll_acc node hd tl l mid i ln :
        dll_chain hd mid tl !! i = Some ln →
        slru_dll node hd tl l -∗
        ∃ kv, ⌜dll_payloads l !! i = Some kv⌝ ∗
          node ln kv (dll_next (dll_chain hd mid tl) i)
                     (dll_prev (dll_chain hd mid tl) i) ∗
          (∀ nx' pv', node ln kv nx' pv' -∗ ...)

      but [mid] is existentially quantified inside [slru_dll], so it has to be
      destructed first rather than passed in. In practice you will want the
      three specialised lemmas below instead of a fully general accessor --
      they are the only three shapes the LRU code actually performs. *)

  (** ** Surgery lemmas

      These correspond one-to-one with the unsafe operations in the Rust code.
      Proofs are left open; each is a straightforward but fiddly induction /
      [big_sepL] rearrangement over [mid]. *)

  (** [LruCache::new]: two freshly boxed sentinels, wired to each other. *)
  Lemma slru_dll_new node hd tl :
    hd ≠ tl →
    hd.(loc_a) ≠ 0 →
    tl.(loc_a) ≠ 0 →
    node hd None tl NULL_loc -∗
    node tl None NULL_loc hd -∗
    slru_dll node hd tl [].
  Proof.
    iIntros (Hne Hhd Htl) "Hhd Htl".
    iExists []. rewrite /dll_chain /dll_payloads /=.
    iSplitR; first done.
    iSplitR.
    { iPureIntro. apply NoDup_cons. split.
      - by intros ?%elem_of_list_singleton.
      - apply NoDup_singleton. }
    iSplitR.
    { iPureIntro. by repeat constructor. }
    iSplitL "Hhd".
    { iExists None. iSplitR; first done. iFrame. }
    iSplitL "Htl"; last done.
    { iExists None. iSplitR; first done. iFrame. }
  Qed.

  (** [ListNode::detach]: unlink the node at payload index [i]. The caller has
      already patched [prev.next] and [next.prev]; what it gets back is the
      shortened list plus sole ownership of the detached node (whose own
      [next]/[prev] fields are stale, hence existentially quantified). *)
  Lemma slru_dll_detach node hd tl l i x :
    l !! i = Some x →
    slru_dll node hd tl l -∗
    ∃ ln nx pv,
      node ln (Some x) nx pv ∗
      (* the neighbours have been repointed past [ln] *)
      slru_dll node hd tl (delete i l).
  Proof. Admitted.

  (** [ListNode::attach] on the head sentinel: splice a node in at the MRU end.
      This is the closing half of [detach] + [attach], i.e. the "touch" that
      [get] and the [Some] branch of [put] perform. *)
  Lemma slru_dll_attach_front node hd tl l ln x nx pv :
    ln ∉ dll_chain hd [] tl →            (* [ln] is fresh w.r.t. the sentinels *)
    ln.(loc_a) ≠ 0 →
    node ln (Some x) nx pv -∗
    slru_dll node hd tl l -∗
    (* after the caller has set [ln.next := hd.next], [ln.prev := hd],
       [hd.next := ln], [old_first.prev := ln] *)
    slru_dll node hd tl (x :: l).
  Proof. Admitted.

  (** Combined form for the "move to front" operation, which is what [get] and
      the [Some] branch of [put] really do. Note that the resulting list is a
      rotation, not an insertion: no allocation happens. *)
  Lemma slru_dll_touch node hd tl l i x :
    l !! i = Some x →
    slru_dll node hd tl l -∗
    slru_dll node hd tl (x :: delete i l).
  Proof. Admitted.

  (** The eviction step in the third branch of [put]: reuse the LRU node's
      allocation with a new payload. [l] is non-empty because [cap > 0] and the
      branch is guarded by [size = cap]. *)
  Lemma slru_dll_evict_reuse node hd tl l x y :
    last l = Some y →
    slru_dll node hd tl l -∗
    slru_dll node hd tl (x :: removelast l).
  Proof. Admitted.

  (** [Drop]: peel the chain off one node at a time from [hd]. *)
  Lemma slru_dll_uncons node hd tl l :
    slru_dll node hd tl l -∗
    ∃ nx, node hd None nx NULL_loc ∗
      (⌜l = [] ∧ nx = tl⌝ ∗ node tl None NULL_loc hd) ∨
      (∃ x l', ⌜l = x :: l'⌝ ∗ slru_dll node nx tl l').
  Proof. Admitted.

End dll.

Global Arguments slru_dll {_ _ _ _} _ _ _ _.

(* ---------------------------------------------------------------------------
   Companion [dune] file, to sit next to this one:

   (rocq.theory
    (flags -w -notation-overridden -w -redundant-canonical-projection)
    (package extra-proofs)
    (name refinedrust.extra_proofs.slru)
    (generate_project_file)
    (theories stdpp iris iris_contrib Ltac2 Equations lrust radium lithium
              refinedrust Stdlib))
   --------------------------------------------------------------------------- *)