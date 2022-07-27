# CosmWasm Smart Contract Workshop

## Design

In the workshop, we would work on a smart contract system built out of two
contracts: the donation peer and the donations manager. The idea is simple: a
group of creators wants a common donation system. They want to share the
donations between each other to increase the donators pool, but at the same
time, peers with more donators should receive more significant parts of
donations.

Everyone is a part of the group has his own `peer` contract. It serves two
purposes: first, it is where donations end up, and the creator withdraws his
donations from this contract. Peer is also an API for donators - anyone can
send funds to the peer to increase the shares of this peer in rewards it
receives. Then part of the funds would stay on the peer contract itself, and
the rest of the donation would be sent to the `manager` contract.

Manager contract is the other part of the system. First of all, it distributes
received donations proportionally to peer contracts. Secondly, it is
responsible for creating peer contracts for creators.

Here is the graph of how contracts are related to each other:

```mermaid

classDiagram
    direction RL
    Manager o-- Peer: Manages
    class Manager {
        join()
        leave()
        donate()
    }
    class Peer {
        donate()
        withdraw()
    }
```

Note: if you find this design a bit suspicious, you are probably right - there
are many problems with it, and in real life, it would probably be designed in a
slightly different way. But this is how I can show some common techniques
easily.

There are four flows in this contract. Let's start with the use-case of a new
creator joining the donation pool: 

```mermaid
sequenceDiagram
Creator->>Manager: Join
Manager->>Peer: Instantiate(config)
Peer->>Manager: Reply(addr)
Manager-->>Manager: Register peer
```

Another very simple flow is the leaving one:

```mermaid
sequenceDiagram
Creator->>Manager: Leave
Manager-->>Manager: Remove peer
```

The clue of the system is the donation flow. Here is what it looks like:

```mermaid
sequenceDiagram

Donator->>Peer1: Donate(funds)

Peer1->>Manager: Donate(funds * collective_ratio)

Manager->>Peer1: QueryDonators
Peer1->>Manager: DonatorsCount
Manager->>Peer2: QueryDonators
Peer2->>Manager: DonatorsCount
Manager->>Peer3: QueryDonators
Peer3->>Manager: DonatorsCount

par Send Peer1 part
    Manager->>Peer1: Send(funds * collective_ratio * peer1_donators / total_donators)
and
    Manager->>Peer2: Send(funds * collective_ratio * peer2_donators / total_donators)
and
    Manager->>Peer3: Send(funds * collective_ratio * peer3_donators / total_donators)
end
```

The last flow to implement is the withdrawal flow. It would not be very complicated:

```mermaid
sequenceDiagram
Creator->>Peer: Withdraw
opt Sender is owner
    Peer->>Bank: Query balance(Peer)
    Bank->>Peer: Balance
    Peer->>Creator: Send(balance)
end
```
