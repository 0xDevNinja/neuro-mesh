# Security Policy

## Overview

Security is a top priority for NeuroMesh. As a decentralized protocol handling value transfer and AI inference, we take the protection of our users, validators, miners, and integrators seriously.

This document outlines our security policies, vulnerability reporting process, and security best practices.

---

## Supported Versions

| Version | Supported          | Notes |
|---------|--------------------|-------|
| 0.x.x   | :white_check_mark: | Active development, security fixes applied |
| < 0.1.0 | :x:                | Pre-release, not recommended for production |

---

## Reporting a Vulnerability

### Disclosure Policy

We follow a **Coordinated Disclosure** policy. Please do not publicly disclose vulnerabilities until we have had a chance to investigate and release a fix.

### How to Report

**For security vulnerabilities, please DO NOT open a public GitHub issue.**

Instead, report vulnerabilities through one of these channels:

1. **Email:** [security@neuromesh.io](mailto:security@neuromesh.io)
2. **Encrypted Email:** Use our PGP key (see below)
3. **GitHub Security Advisories:** [Report a vulnerability](https://github.com/0xDevNinja/neuro-mesh/security/advisories/new)

### PGP Key

```
-----BEGIN PGP PUBLIC KEY BLOCK-----
[PGP key will be added upon project launch]
-----END PGP PUBLIC KEY BLOCK-----
```

### What to Include

When reporting a vulnerability, please include:

1. **Description:** Clear description of the vulnerability
2. **Impact:** Potential impact and severity assessment
3. **Reproduction:** Step-by-step instructions to reproduce
4. **Proof of Concept:** Code or screenshots demonstrating the issue
5. **Suggested Fix:** If you have ideas for remediation
6. **Your Contact:** How we can reach you for follow-up

### Response Timeline

| Action | Timeline |
|--------|----------|
| Initial acknowledgment | 24 hours |
| Preliminary assessment | 72 hours |
| Status update | 7 days |
| Fix development | 14-30 days (severity dependent) |
| Public disclosure | 90 days (or upon fix release) |

---

## Security Bug Bounty Program

### Scope

The following components are in scope for our bug bounty program:

| Component | Repository | Severity Range |
|-----------|------------|----------------|
| NeuroChain Runtime | `src/chain/` | Critical - Low |
| Consensus Mechanism | `src/chain/pallets/` | Critical - Low |
| Node Networking | `src/node/` | High - Low |
| Aggregator API | `src/aggregator/` | High - Low |
| SDK Libraries | `src/sdk/` | Medium - Low |

### Out of Scope

- Third-party dependencies (report upstream)
- Social engineering attacks
- Denial of service attacks
- Issues in non-production environments
- Issues already reported or known

### Severity Classification

| Severity | CVSS Score | Example | Bounty Range |
|----------|------------|---------|--------------|
| **Critical** | 9.0 - 10.0 | Remote code execution, fund theft, consensus manipulation | $10,000 - $50,000 |
| **High** | 7.0 - 8.9 | Privilege escalation, data breach, significant economic loss | $5,000 - $10,000 |
| **Medium** | 4.0 - 6.9 | Information disclosure, limited economic impact | $1,000 - $5,000 |
| **Low** | 0.1 - 3.9 | Minor issues, best practice violations | $100 - $1,000 |

*Bounty amounts are determined based on severity, exploitability, and impact.*

### Eligibility

To be eligible for a bounty:

- Be the first to report the vulnerability
- Follow responsible disclosure guidelines
- Not be a current or former employee
- Not exploit the vulnerability beyond proof of concept
- Provide sufficient detail for reproduction

---

## Security Architecture

### Threat Model

NeuroMesh operates in an adversarial environment. Our threat model considers:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Threat Actors                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐       │
│  │   Malicious   │  │   Malicious   │  │   External    │       │
│  │    Miners     │  │  Validators   │  │   Attackers   │       │
│  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘       │
│          │                  │                  │                │
│          ▼                  ▼                  ▼                │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Attack Vectors                          │ │
│  ├───────────────────────────────────────────────────────────┤ │
│  │ • Sybil attacks (fake identities)                         │ │
│  │ • Collusion (validator cartels)                           │ │
│  │ • Weight manipulation                                     │ │
│  │ • Eclipse attacks (network isolation)                     │ │
│  │ • Front-running (MEV extraction)                          │ │
│  │ • Smart contract exploits                                 │ │
│  │ • API abuse and DoS                                       │ │
│  │ • Key compromise                                          │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Security Controls

#### 1. Consensus Security

| Control | Description | Implementation |
|---------|-------------|----------------|
| Stake Requirements | Minimum stake to participate | Configurable per subnet |
| Slashing | Penalty for misbehavior | Automatic via pallet |
| Reputation System | Trust scoring for validators | Weight-based consensus |
| Cartel Detection | Identify colluding validators | Correlation analysis |

#### 2. Network Security

| Control | Description | Implementation |
|---------|-------------|----------------|
| Peer Authentication | Verify peer identity | libp2p signed messages |
| Transport Encryption | Encrypt all traffic | TLS 1.3 / Noise protocol |
| Rate Limiting | Prevent spam | Token bucket per peer |
| Eclipse Protection | Maintain diverse connections | Kademlia DHT |

#### 3. API Security

| Control | Description | Implementation |
|---------|-------------|----------------|
| Authentication | Verify client identity | API keys / JWT |
| Authorization | Enforce access policies | RBAC middleware |
| Input Validation | Sanitize all inputs | Schema validation |
| Rate Limiting | Prevent abuse | Per-client quotas |

#### 4. Cryptographic Security

| Component | Algorithm | Key Size |
|-----------|-----------|----------|
| Signing | Ed25519 | 256-bit |
| Hashing | Blake2b | 256-bit |
| Encryption | ChaCha20-Poly1305 | 256-bit |
| Key Derivation | Argon2id | N/A |

---

## Security Best Practices

### For Miners

1. **Secure Key Storage**
   - Use hardware security modules (HSM) for production
   - Never store private keys in plaintext
   - Implement key rotation policies

2. **Network Security**
   - Run behind a firewall
   - Use VPN for management access
   - Monitor for unusual traffic patterns

3. **Software Updates**
   - Subscribe to security announcements
   - Apply patches promptly
   - Test updates in staging first

### For Validators

1. **Operational Security**
   - Dedicated hardware for validation
   - Multi-signature for high-value operations
   - Regular security audits

2. **Monitoring**
   - Alert on stake balance changes
   - Monitor for slashing conditions
   - Track reputation score changes

3. **Backup & Recovery**
   - Encrypted backups of validator state
   - Documented recovery procedures
   - Regular recovery drills

### For Integrators

1. **API Security**
   - Store API keys securely (use secrets manager)
   - Rotate keys periodically
   - Use least-privilege access

2. **Input Validation**
   - Validate all responses from NeuroMesh
   - Implement timeout handling
   - Handle errors gracefully

3. **Dependency Management**
   - Pin SDK versions
   - Monitor for security advisories
   - Update dependencies regularly

---

## Security Audits

### Completed Audits

| Date | Auditor | Scope | Report |
|------|---------|-------|--------|
| TBD | TBD | Smart contracts | [Link] |
| TBD | TBD | Consensus mechanism | [Link] |
| TBD | TBD | Network layer | [Link] |

### Planned Audits

- [ ] Pre-mainnet comprehensive audit
- [ ] Annual security review
- [ ] Continuous fuzzing program

---

## Incident Response

### Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| **SEV-1** | Critical, active exploitation | Immediate | Fund theft, consensus attack |
| **SEV-2** | High, potential exploitation | 4 hours | Vulnerability discovered, no active exploit |
| **SEV-3** | Medium, limited impact | 24 hours | Information disclosure, DoS |
| **SEV-4** | Low, minimal impact | 72 hours | Minor bugs, hardening |

### Response Process

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Detect    │────>│   Assess    │────>│   Contain   │────>│   Resolve   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │                   │
       ▼                   ▼                   ▼                   ▼
  • Monitoring        • Severity         • Isolate            • Fix deployed
  • User report       • Impact           • Communicate         • Post-mortem
  • Audit finding     • Scope            • Preserve evidence   • Update docs
```

### Communication

- **Status Page:** [status.neuromesh.io](https://status.neuromesh.io)
- **Security Announcements:** [security@neuromesh.io](mailto:security@neuromesh.io)
- **Discord:** #security-alerts channel

---

## Compliance

### Standards

- OWASP Top 10 Web Application Security Risks
- CWE/SANS Top 25 Most Dangerous Software Errors
- NIST Cybersecurity Framework

### Data Protection

- No personal data stored on-chain
- API logs retained for 30 days
- Right to erasure supported for off-chain data

---

## Security Contacts

| Role | Contact |
|------|---------|
| Security Lead | security@neuromesh.io |
| Emergency Contact | emergency@neuromesh.io |
| Bug Bounty | bounty@neuromesh.io |

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-02-09 | 1.0 | Initial security policy |

---

*This security policy is reviewed and updated quarterly.*

*Last updated: 2026-02-09*
