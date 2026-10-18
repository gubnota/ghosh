use ghosh_core::{approvals::*, contracts::ErrorCode, policy::Principal};

fn binding() -> ApprovalBinding {
    ApprovalBinding {
        context: ApprovalContext {
            workspace_id: ulid::Ulid::new().to_string(),
            revision: content_digest(b"manifest"),
            session_id: uuid::Uuid::new_v4().to_string(),
        },
        requester: Requester::Mcp,
        tool: "node.run".into(),
        source_digest: content_digest(b"reviewed source"),
        targets: vec!["edge1".into(), "edge2".into()],
        route_digest: content_digest(b"direct route"),
        identity_digest: content_digest(b"explicit registry key generation 1"),
    }
}
fn queue(binding: &ApprovalBinding) -> (Approvals, String) {
    let mut service = Approvals::default();
    service.synchronize(Some(binding.context.clone()));
    let id = service
        .request(Principal::NativeService, binding.clone())
        .unwrap();
    (service, id)
}
fn decide(service: &mut Approvals, id: &str, decision: ApprovalDecision) {
    service
        .decide(
            Principal::LocalUi,
            ApprovalDecisionRequest {
                id: id.into(),
                decision,
            },
        )
        .unwrap();
}

#[test]
fn denied_pending_and_replayed_requests_have_zero_effects() {
    let binding = binding();
    let (mut service, id) = queue(&binding);
    let mut effects = 0;
    if service
        .consume(Principal::NativeService, &id, &binding)
        .is_ok()
    {
        effects += 1;
    }
    decide(&mut service, &id, ApprovalDecision::Deny);
    if service
        .consume(Principal::NativeService, &id, &binding)
        .is_ok()
    {
        effects += 1;
    }
    assert_eq!(effects, 0);
    assert!(
        service
            .decide(
                Principal::LocalUi,
                ApprovalDecisionRequest {
                    id: id.clone(),
                    decision: ApprovalDecision::AllowOnce
                }
            )
            .is_err()
    );
    let (mut service, id) = queue(&binding);
    decide(&mut service, &id, ApprovalDecision::AllowOnce);
    for _ in 0..2 {
        if service
            .consume(Principal::NativeService, &id, &binding)
            .is_ok()
        {
            effects += 1;
        }
    }
    assert_eq!(effects, 1);
    assert_eq!(
        service.snapshot().requests[0].state,
        ApprovalState::Consumed
    );
}

#[test]
fn every_execution_identity_component_is_bound() {
    let binding = binding();
    let (mut service, id) = queue(&binding);
    decide(&mut service, &id, ApprovalDecision::AllowOnce);
    for changed in 0..9 {
        let mut stale = binding.clone();
        match changed {
            0 => stale.context.workspace_id = ulid::Ulid::new().to_string(),
            1 => stale.context.revision = content_digest(b"new config"),
            2 => stale.context.session_id = uuid::Uuid::new_v4().to_string(),
            3 => stale.source_digest = content_digest(b"changed script"),
            4 => stale.targets.pop().map(|_| ()).unwrap(),
            5 => stale.route_digest = content_digest(b"changed route"),
            6 => stale.identity_digest = content_digest(b"relinked key"),
            7 => stale.requester = Requester::Script,
            _ => stale.tool = "node.exec".into(),
        }
        assert_eq!(
            service
                .consume(Principal::NativeService, &id, &stale)
                .unwrap_err()
                .code,
            ErrorCode::Denied
        );
    }
    // Failed attempts cannot consume the correct user's one-use approval.
    service
        .consume(Principal::NativeService, &id, &binding)
        .unwrap();
}

#[test]
fn exact_session_grants_are_reusable_revocable_and_reset_on_context_change() {
    let binding = binding();
    let (mut service, id) = queue(&binding);
    decide(&mut service, &id, ApprovalDecision::AllowSession);
    service
        .consume(Principal::NativeService, &id, &binding)
        .unwrap();
    service
        .consume(Principal::NativeService, &id, &binding)
        .unwrap();
    decide(&mut service, &id, ApprovalDecision::Revoke);
    assert!(
        service
            .consume(Principal::NativeService, &id, &binding)
            .is_err()
    );
    for reason in 0..4 {
        let (mut service, id) = queue(&binding);
        decide(&mut service, &id, ApprovalDecision::AllowSession);
        let mut context = binding.context.clone();
        match reason {
            0 => context.workspace_id = ulid::Ulid::new().to_string(),
            1 => context.revision = content_digest(b"rename"),
            _ => context.session_id = uuid::Uuid::new_v4().to_string(),
        }
        service.synchronize(if reason == 3 { None } else { Some(context) });
        assert_eq!(
            service.snapshot().requests[0].state,
            ApprovalState::Invalidated
        );
        assert!(
            service
                .consume(Principal::NativeService, &id, &binding)
                .is_err()
        );
        service.synchronize(Some(binding.context.clone()));
        assert!(
            service
                .consume(Principal::NativeService, &id, &binding)
                .is_err()
        );
    }
}

#[test]
fn callers_cannot_forge_requests_decisions_or_consumption() {
    let binding = binding();
    let (mut service, id) = queue(&binding);
    for caller in [Principal::LocalUi, Principal::Mcp, Principal::Script] {
        assert!(service.request(caller, binding.clone()).is_err());
    }
    for caller in [Principal::NativeService, Principal::Mcp, Principal::Script] {
        assert!(
            service
                .decide(
                    caller,
                    ApprovalDecisionRequest {
                        id: id.clone(),
                        decision: ApprovalDecision::AllowOnce
                    }
                )
                .is_err()
        );
    }
    decide(&mut service, &id, ApprovalDecision::AllowOnce);
    for caller in [Principal::LocalUi, Principal::Mcp, Principal::Script] {
        assert!(service.consume(caller, &id, &binding).is_err());
    }
    let mut malformed = binding.clone();
    malformed.targets.reverse();
    assert!(
        service
            .request(Principal::NativeService, malformed)
            .is_err()
    );
    let mut malformed = binding;
    malformed.source_digest = "not a digest".into();
    assert!(
        service
            .request(Principal::NativeService, malformed)
            .is_err()
    );
}

#[test]
fn bounded_requests_reject_overload_without_losing_pending_decisions() {
    let binding = binding();
    let (mut service, first) = queue(&binding);
    for _ in 1..128 {
        service
            .request(Principal::NativeService, binding.clone())
            .unwrap();
    }
    assert_eq!(
        service
            .request(Principal::NativeService, binding.clone())
            .unwrap_err()
            .code,
        ErrorCode::Capacity
    );
    decide(&mut service, &first, ApprovalDecision::Deny);
    service.request(Principal::NativeService, binding).unwrap();
    let snapshot = service.snapshot();
    assert_eq!(snapshot.requests.len(), 128);
    assert_eq!(snapshot.audit.len(), 130);
}
