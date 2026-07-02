# References Index

Generated: 2026-06-16 — 50 papers across 11 themes.
Re-run `build_index.py` after adding or moving papers.

---

## Table of Contents

- [Aws](#aws) (2 papers)
- [Hpc@Cloud](#hpc@cloud) (2 papers)
- [Manaanddmtcp](#manaanddmtcp) (4 papers)
- [Mpi And Parallel Programing](#mpi-and-parallel-programing) (4 papers)
- [Burstable And Spot Instances](#burstable-and-spot-instances) (4 papers)
- [Cloud Computing](#cloud-computing) (12 papers)
- [Cost Study](#cost-study) (3 papers)
- [Fault Tolerance](#fault-tolerance) (14 papers)
- [Nas Parallel Benchmarks](#nas-parallel-benchmarks) (1 paper)
- [Project Documents](#project-documents) (1 paper)
- [Scientific Initiation](#scientific-initiation) (3 papers)

---

## Aws

### What is Amazon Elastic File System
**Full text:** [What is Amazon Elastic File System.txt](references/AWS/What is Amazon Elastic File System.txt)

Amazon Elastic File System (Amazon EFS) provides serverless, fully elastic file storage so that you
can share file data without provisioning or managing storage capacity and performance. Amazon EFS is
built to scale on demand to petabytes without disrupting applications, growing and shrinking
automatically as you add and remove files. Because Amazon EFS has a simple web services interface,
you can create and configure file systems quickly and easily. The service manages all the file
storage infrastructure for you, meaning that you can avoid the complexity of deploying, patching,
and maintaining complex file system configurations. Amazon EFS supports the Network File System
version 4 (NFSv4.1 and NFSv4.0) protocol, so the applications and tools that you use today work
seamlessly with Amazon EFS.

---

### s10586 023 04060 4
**Full text:** [s10586-023-04060-4.txt](references/AWS/s10586-023-04060-4.txt)

Cloud computing platforms have been continuously evolving. Features such as the Elastic Fabric
Adapter (EFA) in the Amazon Web Services (AWS) platform have brought yet another revolution in the
High Performance Computing (HPC) world, further accelerating the convergence of HPC and cloud
computing. Other public clouds also support similar features further fueling this change. In this
paper, we show how and why the performance of a large-scale computational ﬂuid dynamics (CFD) HPC
application on AWS competes very closely with the one on Beskow—a Cray XC40 supercomputer at the PDC
Center for High-Performance Computing - in terms of cost-efﬁciency with strong scaling up to 2304
processes. We perform an extensive set of micro and macro benchmarks in both environments and
conduct a comparative analysis.

---

## Hpc@Cloud

### Enabling the execution of HPC applications  n public clouds with HPC Cloud
**Full text:** [Enabling_the_execution_of_HPC_applications _n_public_clouds_with_HPC_Cloud.txt](references/HPC@Cloud/Enabling_the_execution_of_HPC_applications _n_public_clouds_with_HPC_Cloud.txt)

The advent of cloud computing has made access to computing infrastructure available to millions of
users that face resource constraints. In the context of high performance computing (HPC), public
cloud resources have emerged as a cost-effective alternative to expensive on-premises clusters.
However, there are several challenges and limitations in adopting this approach. This paper proposes
HPC@Cloud , a provider-agnostic open-source software toolkit that facilitates the migration,
testing, and execution of HPC applications in public clouds. The toolkit takes advantage of various
fault tolerance technologies to enable the use of inex- pensive transient cloud infrastructure,
commonly known as “spot” instances.

---

### HPC@Cloud: A Provider Agnostic Toolkit to Enable the Execution of HPC Applications on Public Clouds
**Full text:** [HPC@Cloud:_A_Provider-Agnostic_Toolkit_to_Enable_the_Execution_of_HPC_Applications_on_Public_Clouds.txt](references/HPC@Cloud/HPC@Cloud:_A_Provider-Agnostic_Toolkit_to_Enable_the_Execution_of_HPC_Applications_on_Public_Clouds.txt)

The advent of cloud computing has made access to computing infrastructure available to millions of
researchers and organizations. In the context of High-Performance Computing (HPC), public cloud
resources have emerged as a cost-eﬀective alternative to expensive on-premises clusters. However,
there are several challenges and limitations in adopting this approach. This dissertation proposes
HPC@Cloud, a multi-provider, open-source software toolkit that facilitates the migration, testing,
and execution of HPC applications on public cloud platforms. The toolkit leverages various fault
tolerance technologies to enable the use of inexpensive ephemeral cloud infrastructure, commonly
known as “spot” instances in Amazon Web Services (AWS).

---

## Manaanddmtcp

### 2408.02218v1
**Full text:** [2408.02218v1.txt](references/MANAandDMTCP/2408.02218v1.txt)

MPI is the de facto standard for parallel computing on a cluster of computers. Checkpointing is an
important component in any strategy for software resilience and for long-running jobs that must be
executed by chaining together time-bounded resource allocations. This work solves an old problem: a
practical and general algorithm for transparent checkpointing of MPI that is both efficient and
compatible with most of the latest network software. Transparent checkpointing is attractive due to
its generality and ease of use for most MPI application developers. Earlier efforts at transparent
checkpointing for MPI, one decade ago, had two difficult problems: (i) by relying on a specific MPI
implementation tied to a specific network technology; and (ii) by failing to demonstrate
sufficiently low runtime overhead.

---

### 3624062.3624255
**Full text:** [3624062.3624255.txt](references/MANAandDMTCP/3624062.3624255.txt)

This work presents experience with traditional use cases of check- pointing on a novel platform. A
single codebase (MANA) transpar- ently checkpoints production workloads for major available MPI
implementations: “develop once, run everywhere”. The new plat- form enables application developers
to compile their application against any of the available standards-compliant MPI implementa- tions,
and test each MPI implementation according to performance or other features. Since its original
academic prototype, MANA has been under development for three of the past four years, and is planned
to enter full production at NERSC in early Fall of 2023. To the best of the authors’ knowledge, MANA
is currently the only production- capable, system-level checkpointing package running on a large
supercomputer (Perlmutter at NERSC) using a major MPI imple- mentation (HPE Cray MPI).

---

### dmtcp
**Full text:** [dmtcp.txt](references/MANAandDMTCP/dmtcp.txt)

DMTCP (Distributed MultiThreaded CheckPointing) is a transparent user-level checkpointing package
for distributed applications. Checkpointing and restart is demonstrated for a wide range of over 20
well known applications, includ- ing MATLAB, Python, TightVNC, MPICH2, OpenMPI, and runCMS. RunCMS
runs as a 680 MB image in memory that includes 540 dynamic libraries, and is used for the CMS
experiment of the Large Hadron Collider at CERN. DMTCP transparently checkpoints general cluster
compu- tations consisting of many nodes, processes, and threads; as well as typical desktop
applications. On 128 distributed cores (32 nodes), checkpoint and restart times are typically 2
seconds, with negligible run-time overhead.

---

### hpdc19
**Full text:** [hpdc19.txt](references/MANAandDMTCP/hpdc19.txt)

Transparently checkpointing MPI for fault tolerance and load bal- ancing is a long-standing problem
in HPC. The problem has been complicated by the need to provide checkpoint-restart services for all
combinations of an MPI implementation over all network interconnects. This work presents MANA (MPI-
Agnostic Network- Agnostic transparent checkpointing), a single code base which sup- ports all MPI
implementation and interconnect combinations. The agnostic properties imply that one can checkpoint
an MPI appli- cation under one MPI implementation and perhaps over TCP, and then restart under a
second MPI implementation over InfiniBand on a cluster with a different number of CPU cores per
node. This tech- nique is based on a novel split-process approach, which enables two separate
programs to co-exist within a single process with a single address space.

---

## Mpi And Parallel Programing

### 19170 361 15324 1 10 20220322 1
**Full text:** [19170-361-15324-1-10-20220322-1.txt](references/MPI-and-parallel-programing/19170-361-15324-1-10-20220322-1.txt)

. This ongoing study presents an analysis of the scaling of MPI-enabled  partial differential
equations (PDE) systems solver implementations with public  cloud infrastructure. Shallow-Water
equations are used as case study for analy-  sis. Results indicate that public cloud can be a cost-
effective solution  for highly  coupled problems, given a certain problem size threshold and
appropriate in-  frastructure configuration. However, adequate software tooling and models are
required for estimating and dimensioning optimal clusters.

---

### hursey2020
**Full text:** [hursey2020.txt](references/MPI-and-parallel-programing/hursey2020.txt)

Packaging parallel applications in containers has become increasingly popular on High Performance
Computing (HPC) systems. These applications depend on the Message Pass- ing Interface (MPI) for
communication between processes in their parallel jobs. This paper explores the design
considerations that container maintainers must weigh when building and running their applications on
HPC systems. This includes highlighting and resolving the cgroup, namespace, and security boundaries
used by some container runtimes that may hinder an MPI library from performing efﬁciently. This
paper explores the impact on MPI libraries of two opposing container launch models, and various
models for incorporating a system optimized MPI library including a novel hybrid BYO-MPI with system
mounted components technique.

---

### mpi41 report
**Full text:** [mpi41-report.txt](references/MPI-and-parallel-programing/mpi41-report.txt)

interfaces, or modules, beginning with the prefix MPI_. To avoid conflicting with the profiling
interface, programs must also avoid subroutines and functions with the prefix PMPI_. This is
mandated to avoid possible name collisions. All MPI Fortran subroutines have an error code in the
last argument. With USE mpi_f08, this last argument is declared as OPTIONAL, except for user-defined
callback functions (e.g., COMM_COPY_ATTR_FUNCTION) and their predefined callbacks (e.g.,
MPI_COMM_NULL_COPY_FN).

---

### slurm
**Full text:** [slurm.txt](references/MPI-and-parallel-programing/slurm.txt)

A new cluster resource management system called Simple Linux Utility Resource Management (SLURM) is
developed and presented in this paper. SLURM, initially developed for large Linux clusters at the
Lawrence Livermore National Laboratory (LLNL), is a simple cluster manager that can scale to
thousands of processors. SLURM is designed to be ﬂexible and fault-tolerant and can be ported to
other clusters of diﬀer- ent size and architecture with minimal eﬀort. We are certain that SLURM
will beneﬁt both users and system architects by providing them with a simple, robust, and highly
scalable parallel job execution environment for their cluster system. 1 Introduction Linux clusters,
often constructed by using commodity oﬀ-the-shelf (COTS) componnets, have become increasingly
populuar as a computing platform for parallel computation in recent years, mainly due to their
ability to deliver high perfomance-cost ratio.

---

## Burstable And Spot Instances

### Guide burstable performance instances
**Full text:** [Guide-burstable-performance-instances.txt](references/burstable-and-spot-instances/Guide-burstable-performance-instances.txt)

Many general purpose workloads are on average not busy, and do not require a high level of sustained
CPU performance. The following graph illustrates the CPU utilization for many common workloads that
customers run in the AWS Cloud today. These low-to-moderate CPU utilization workloads lead to
wastage of CPU cycles and, as a result, you pay for more than you use. To overcome this, you can
leverage the low-cost burstable general purpose instances, which are the T instances. The T instance
family provides a baseline CPU performance with the ability to burst above the baseline at any time
for as long as required.

---

### New Amazon EC2 Spot pricing model  Simplified purchasing without bidding and fewer interruptions   AWS Compute Blog
**Full text:** [New Amazon EC2 Spot pricing model_ Simplified purchasing without bidding and fewer interruptions _ AWS Compute Blog.txt](references/burstable-and-spot-instances/New Amazon EC2 Spot pricing model_ Simplified purchasing without bidding and fewer interruptions _ AWS Compute Blog.txt)

Amazon EC2 Spot Instances oﬀer spare compute capacity in the AWS Cloud at steep discounts.
Customers— including Yelp, NASA JPL, FINRA, and Autodesk—use Spot Instances to reduce costs and get
faster results. Spot Instances provide acceleration, scale, and deep cost savings to big data
workloads, containerized applications such as web services, test/dev, and many types of HPC and
batch jobs. At re:Invent 2017, we launched a new pricing model that simpliﬁed the Spot purchasing
experience. The new model gives you predictable prices that adjust slowly over days and weeks, with
typical savings of 70-90% over On-Demand.

---

### Using Burstable Instances in the Public Cloud: Why When and How
**Full text:** [Using_Burstable_Instances_in_the_Public_Cloud:_Why_When_and_How.txt](references/burstable-and-spot-instances/Using_Burstable_Instances_in_the_Public_Cloud:_Why_When_and_How.txt)

Using Burstable Instances in the Public Cloud: Why, When and How? Amazon EC2 and Google Compute
Engine (GCE) have recently introduced a new class of virtual machines called “burstable” instances
that are cheaper than even the smallest traditional/regular instances. These lower prices come with
reduced average capacity and increased variance. Using measurements from both EC2 and GCE, we
identify key idiosyncrasies of resource capacity dynamism for burstable instances that set them
apart from other instance types. Most importantly, certain resources for these instances appear to
be regulated by deterministic token bucket like mechanisms.

---

### spot instance termination notices
**Full text:** [spot-instance-termination-notices.txt](references/burstable-and-spot-instances/spot-instance-termination-notices.txt)

A Spot Instance interruption notice is a warning that is issued two minutes before Amazon EC2 stops
or terminates your Spot Instance. If you specify hibernation as the interruption behavior, you
receive an interruption notice, but you do not receive a two-minute warning because the hibernation
process begins immediately. The best way for you to gracefully handle Spot Instance interruptions is
to architect your application to be fault-tolerant. To accomplish this, you can take advantage of
Spot Instance interruption notices. We recommend that you check for these The interruption notices
are made available as an EventBridge event and as items in the instance metadata on the Spot
Instance.

---

## Cloud Computing

### 2011.00656v2
**Full text:** [2011.00656v2.txt](references/cloud-computing/2011.00656v2.txt)

Can cloud computing infrastructures provide HPC-competitive per- formance for scientific
applications broadly? Despite prolific re- lated literature, this question remains open. Answers are
crucial for designing future systems and democratizing high-performance computing. We present a
multi-level approach to investigate the performance gap between HPC and cloud computing, isolating
dif- ferent variables that contribute to this gap. Our experiments are divided into (i) hardware and
system microbenchmarks and (ii) user application proxies.

---

### 3150224
**Full text:** [3150224.txt](references/cloud-computing/3150224.txt)

details can increase user productivity. Execution steering (Ocaña et al. 2011; de Oliveira et al.
2010) Related to automatic creation of jobs with different input parameters. Facilitation of
parameter sweeping experiments for non-IT specialists.

---

### 3241737
**Full text:** [3241737.txt](references/cloud-computing/3241737.txt)

the low-level technicalities away from the user. Therefore, it becomes nec- essary that the hardware
is abstracted via a middleware for applications to exploit. Certainly, this comes at the expense of
performance and fewer opportunities to optimise the code. Hence, there is a tradeoff between
performance and ease of use, when moving from VMs at the infrastructure level and on to using
software and services available higher up in the computing stack. One open challenge in this area is
developing software that is agnostic of the underlying hardware and can adapt based on the available
hardware [100].

---

### CLOUD COMPUTING Principles and Paradigms
**Full text:** [CLOUD COMPUTING Principles and Paradigms.txt](references/cloud-computing/CLOUD COMPUTING Principles and Paradigms.txt)

hypervisor speciﬁc calls. Storage Virtualization. Virtualizing storage means abstracting logical
sto- rage from physical storage. By consolidating all available storage devices in a data center, it
allows creating virtual disks independent from device and location. Storage devices are commonly
organized in a storage area network (SAN) and attached to servers via protocols such as Fibre
Channel, iSCSI, and 18 INTRODUCTION TO CLOUD COMPUTING

---

### ENABLE HIGH PERFORMANCE COMPUTING IN CLOUD A REVIE
**Full text:** [ENABLE_HIGH_PERFORMANCE_COMPUTING_IN_CLOUD_A_REVIE.txt](references/cloud-computing/ENABLE_HIGH_PERFORMANCE_COMPUTING_IN_CLOUD_A_REVIE.txt)

High Performance Computing (HPC) resulting whole computing power in a way that delivers much higher
performance than it could get in typical  desktop computer or workstation. High Performance
Computing (HPC) allows scientists and engineers to solve complex science, engineering, and  business
problems using applications that require high bandwidth, enhanced networking, and very high compute
capabilities. HPC's  democratization has been driven particularly by cloud computing, which has
given scientists access to supercomputing-like features as the pay as  you go. This paper will
provide an overview on benets, challenges and future of HPC in cloud. KEYWORDS IaaS, SaaS, PaaS,
HaaS, HPC 44 International Journal of Scientiﬁc Research

---

### Efficiency or Innovation The Long Run Payoff of
**Full text:** [Efficiency-or-Innovation__-The-Long-Run-Payoff-of.txt](references/cloud-computing/Efficiency-or-Innovation__-The-Long-Run-Payoff-of.txt)

Considering the mixed arguments and uncertainty about the payoff of cloud computing, this paper
empirically studies the long-term cloud computing impact on the financial performance, specifically
from the perspective of efficiency and innovation. Taking 253 pairs of listed companies in China as
the research sample, propensity score matching and difference in differences techniques combined
with OLS regression are conducted to analyze a rolling 5-year panel data. The analysis results show
that cloud computing adoption leads to years of financial performance decline followed by an upturn.
The downward trend is more pronounced when it is adopted with innovation. This paper contributes  to
the existing literature by leveraging archival performance data to verify the long-term business
value and revealing the value realization difference between efficiency- and innovation-oriented
cloud  computing adoptions.

---

### Gartner Says Worldwide IaaS Public Cloud Services Market Grew 41.4  in 2021
**Full text:** [Gartner Says Worldwide IaaS Public Cloud Services Market Grew 41.4_ in 2021.txt](references/cloud-computing/Gartner Says Worldwide IaaS Public Cloud Services Market Grew 41.4_ in 2021.txt)

The worldwide infrastructure as a service (IaaS) market grew 41.4% in 2021 to total $90.9 billion
up from $64.3 billion in 2020 according to Gartner Inc. Amazon retained the No. 1 position in the
IaaS market in 2021 followed by The IaaS market continues to grow unabated as cloud-native
(https://www.gartner.com/en/newsroom/press- releases/2021-11-10-gartner-says-cloud-will-be-the-
centerpiece-of-new-digital-experiences) becomes the primary architecture for modern workloads said
Sid Nag (https://www.gartner.com/en/experts/sid-nag) VP analyst at Gartner. Cloud supports the
scalability and composability (https://www.gartner.com/en/newsroom/press-
releases/2021-10-18-gartner-survey-of-over-2000-cios-reveals-the-need-for-enterprises-to-embrace-
business- composability-in-2022) that advanced technologies and applications require while also
enabling enterprises to address emerging needs such as sovereignty data integration and enhanced
customer experience. In 2021 the top ive IaaS providers accounted for over 80% of the market.
Amazon continued to lead the worldwide IaaS market with revenue of $35.4 billion in 2021 and 38.9%
market share (see Table 1).

---

### The Evolution of the Cloud
**Full text:** [The_Evolution_of_the_Cloud.txt](references/cloud-computing/The_Evolution_of_the_Cloud.txt)

Cloud computing has enabled the deployment of systems at scale without  requiring deep expertise in
infrastructure management or highly specialized personnel. In just a few years, cloud computing has
become one of the fastest growing technology  segments in the Information Technology industry and it
has transformed how  applications are created and how companies they manage their growth. The cloud
market has quickly become one of the most competitive industries with companies  committing their
efforts to the creation of cloud platforms and aggressive pricing  strategies in an attempt to gain
market dominance. This work shows the origins of the Infrastructure-as-a-Service industry and an
analysis of the market dynamics by looking at the portfolios and strategies of the top  competitors
in this space. Also, this report shows what are the developments that will  drive the innovation in
the cloud industry years to come.

---

### aljamal2018
**Full text:** [aljamal2018.txt](references/cloud-computing/aljamal2018.txt)

This paper reviews the top leading cloud  providers, surveys their offering related to High
Performance  Computing (HPC). Four top cloud provider are selected,  Microsoft Windows Azure, Amazon
Elastic Compute Cloud  (Amazon EC2), Google Compute Engine and Oracle Cloud. Each  one of them has
its unique value proposition that enables it to  survive in the market and make it a real
competitor. A  comparative analysis of the offerings and relative benefits of each  are provided.
This study is an introduction to our extensive work  in the HPC field.

---

### csit64803
**Full text:** [csit64803.txt](references/cloud-computing/csit64803.txt)

Cloud computing has become the ubiquitous computing and storage paradigm. It is also  attractive for
scientists, because they do not have to care any more for their own IT  infrastructure, but can
outsource it to a Cloud Service Provider of their choice. However, for  the case of High-Performance
Computing (HPC) in a cloud, as it is needed in simulations or for  Big Data analysis, things are
getting more intricate, because HPC codes must stay highly  efficient, even when executed by many
virtual cores (vCPUs). Older clouds or new standard  clouds can fulfil this only under special
precautions, which are given in this article. The results  can be extrapolated to other cloud OSes
than OpenStack and to other codes than OpenFOAM,  which were used as examples.

---

### nistspecialpublication800 145
**Full text:** [nistspecialpublication800-145.txt](references/cloud-computing/nistspecialpublication800-145.txt)

The Information Technology Laboratory (ITL) at the National Institute of Standards and Technology
(NIST) promotes the U.S. economy and public welfare by providing technical leadership for the
nation’s measurement and standards infrastructure. ITL develops tests, test methods, reference data,
proof of concept implementations, and technical analysis to advance the development and productive
use of information technology. ITL’s responsibilities include the development of technical,
physical, administrative, and management standards and guidelines for the cost-effective security
and privacy of sensitive unclassified information in Federal computer systems. This Special
Publication 800-series reports on ITL’s research, guidance, and outreach efforts in computer
security and its collaborative activities with industry, government, and academic organizations.

---

### rsta.2019.0061
**Full text:** [rsta.2019.0061.txt](references/cloud-computing/rsta.2019.0061.txt)

machine model, which would break our current programming systems. There is a journal article by Unat
et al. [21] from the PADAL (Programming Abstractions for Data Locality) workshop [22] that outlines
the current state of the art in data locality management in modern programming systems and identiﬁes
numerous opportunities to greatly improve automation in these areas. New algorithms favouring less
data movement or higher arithmetic intensity, such as communication-avoiding and high-order
operators, are already being developed, and data- centric programming abstractions must be built
into new partitioned global address space (PGAS) programming systems in order to confer algorithmic
information about data locality to the underlying software system. These capabilities are even more
crucial for heterogeneous architectures where different accelerators have different
memory/communication speeds.

---

## Cost Study

### APC200026
**Full text:** [APC200026.txt](references/cost-study/APC200026.txt)

. Cloud Computing has emerged as an interesting alternative for running business applications, but
this might not be true for scientiﬁc applications. A com- parison between HPC systems and cloud
infrastructure not always sees the lat- ter winning over the former, especially when only
performance and economical aspects are taken into account. But if other factors, such as turnaround
time and user preference, come into play, the landscape of the usage convenience changes. Choosing
the right infrastructure, then, can be essentially seen as a multi-attribute decision-making
problem.

---

### BoT
**Full text:** [BoT.txt](references/cost-study/BoT.txt)

Cloud providers offer several types of Virtual Machines (VMs) in diverse markets, with different
guarantees in terms of availability and reliability. Among them, the most popular market models are
the on-demand and the spot. On-demand VMs are allocated for a ﬁxed cost per time, and their
availability is ensured during the whole execution. On the other hand, in the spot market, VMs are
offered with a huge discount, but their availability ﬂuctuates according to cloud’s current demand
that can terminate or hibernate a spot VM at any time. Furthermore, to cope with workload
variations, cloud providers have also introduced the concept of burstable VMs, which can burst up
their CPU performance during a limited period of time.

---

### FarSpot TPDS22
**Full text:** [FarSpot-TPDS22.txt](references/cost-study/FarSpot-TPDS22.txt)

Recently, we have witnessed many HPC applications developed and hosted in the cloud, which can
beneﬁt from the elastic and diversiﬁed resources on the cloud, while on the other hand confronting
high costs for executing the long-running HPC applications. Although public clouds such as Amazon
EC2 offer spot instances with dynamic and usually low prices compared to on-demand ones, the spot
prices can vary signiﬁcantly and sometimes can even be more expensive than on-demand prices of the
same type. Previous work on reducing the monetary cost for HPC applications using spot instances
focused on designing fault tolerance techniques or selecting appropriate instance types/bid prices
to make good usage of the low spot prices. However, with the recent update of spot pricing model on
Amazon EC2, these work may become either inefﬁcient or invalid. In this paper, we present FarSpot
which is an optimization framework for HPC applications in the latest cloud spot market with the
goal of minimizing application cost while ensuring performance constraints.

---

## Fault Tolerance

### 2004 aviz laprie randell
**Full text:** [2004-aviz-laprie-randell.txt](references/fault-tolerance/2004-aviz-laprie-randell.txt)

This paper gives the main definitions relating to dependability, a generic concept including as
special case such attributes as reliability, availability, safety, integrity, maintainability, etc.
Security brings in concerns for confidentiality, in addition to availability and integrity. Basic
definitions are given first. They are then commented upon, and supplemented by additional
definitions, which address the threats to dependability and security (faults, errors, failures),
their attributes, and the means for their achievement (fault prevention, fault tolerance, fault
removal, fault forecasting). The aim is to explicate a set of general concepts, of relevance across
a wide range of situations and, therefore, helping communication and cooperation among a number of
scientific and technical communities, including ones that are concentrating on particular types of
system, of system failures, or of causes of system failures.

---

### 568522.568525
**Full text:** [568522.568525.txt](references/fault-tolerance/568522.568525.txt)

values under the control of the recovery system. Also, ﬁle access could be made highly available by
placing all ﬁles in network-wide highly avail- able ﬁle servers or by using dual-ported disks.
Another problem in implementing re- covery is the need to reconstruct the auxil- iary state within
the operating system ker- nel that supports the recovering process [Elnozahy 1993; Huang and Kintala
1993; Johnson 1989; Plank 1993]. This state is usually outside of the recovery protocol’s control
during failure-free operation, un- less the protocol is implemented inside the operating system. For
protocols im- plemented outside the operating system, the rollback-recovery system must emu- late
these data structures and log sufﬁ- cient information to be able to recreate them during recovery.

---

### FT MPI MinicWSCAD2017
**Full text:** [FT-MPI-MinicWSCAD2017.txt](references/fault-tolerance/FT-MPI-MinicWSCAD2017.txt)

O MPI é um dos principais padrões para o desenvolvimento de aplicações para- lelas e distribuídas
baseadas no paradigma de troca de mensagens. Diversos sistemas de computação de alto desempenho são
baseados em MPI. Um dos maiores desaﬁos dos sistemas de alto desempenho diz respeito à
conﬁabilidade, ou seja, à capacidade de ofere- cer serviços corretos ininterruptamente. Este
minicurso tem como objetivo apresentar as principais técnicas empregadas para a construção de
sistemas MPI tolerantes a falhas. São apresentadas técnicas tradicionais aplicadas a sistemas
baseados em MPI, como a técnica rollback-recovery, incluindo suas variantes baseadas em checkpoints
e em regis- tro de mensagens.

---

### IST2020 3299 Ghavamipour
**Full text:** [IST2020-3299-Ghavamipour.txt](references/fault-tolerance/IST2020-3299-Ghavamipour.txt)

IT infrastructures are rapidly growing due to the  increased demand for computing power used by
applications. Furthermore, modern cloud data centers are hosting various  advanced applications
based on the user's needs. The goal of  this paper is to maximize the reliability of running
workflow  applications considering spot instances revocation without  imposing fault-tolerance
overhead. For this purpose, we use an  Artificial Neural Network algorithm (ANN) to define a failure
prediction module for the cloud spot instances. Indeed, we  introduce a novel workflow scheduling
algorithm, named  Reliability Aware and modified HEFT (Heterogeneous Earliest  Finish Time) for
minimizing the makespan of a given workflow  subject to a specified reliability of the application.

---

### amoon2018
**Full text:** [amoon2018.txt](references/fault-tolerance/amoon2018.txt)

The likelihood of failures rises in cloud computing systems as a result of their unstable nature.
Additionally, the size of  a cloud computing system varies with time and thus failures become a
common incident. Failures have a high impact on  cloud performance and the expected benefits for
both customers and providers. Fault tolerance is an essential challenge  facing cloud providers in
order to mitigate the effects of failures and maintaining the Service Level Agreement (SLA) satis‑
fied. Checkpointing is one of the most known reactive fault tolerance techniques used in distributed
computing.

---

### berkeley lab checkpoint restart blcr for linux clusters 3thh14s0g9
**Full text:** [berkeley-lab-checkpoint-restart-blcr-for-linux-clusters-3thh14s0g9.txt](references/fault-tolerance/berkeley-lab-checkpoint-restart-blcr-for-linux-clusters-3thh14s0g9.txt)

. This article describes the motivation, design and implementation of Berkeley Lab
Checkpoint/Restart (BLCR), a system-level checkpoint/restart implementation for Linux  clusters that
targets the space of typical High Performance Computing applications, including  MPI. Application-
level solutions, including both checkpointing and fault-tolerant algorithms,  are recognized as more
time and space efficient than system-level checkpoints, which cannot  make use of any application-
specific knowledge. However, system-level checkpointing allows  for preemption, making it suitable
for responding to “fault precursors” (for instance, elevated  error rates from ECC memory or network
CRCs, or elevated temperature from sensors). Preemption can also increase the efficiency of batch
scheduling; for instance reducing idle  cycles (by allowing for shutdown without any queue draining
period or reallocation of  resources to eliminate idle nodes when better fitting jobs are queued),
and reducing the average  queued time (by limiting large jobs to running during off-peak hours,
without the need to limit  the length of such jobs).

---

### bland2013
**Full text:** [bland2013.txt](references/fault-tolerance/bland2013.txt)

As supercomputers are entering an era of massive parallelism where the frequency of faults is
increasing, the MPI Standard remains distressingly vague on the consequence of failures on MPI
communications. Advanced fault-tolerance techniques have the potential to prevent full-scale
application restart and therefore lower the cost incurred for each failure, but they demand from MPI
the capability to detect failures and resume communications afterward. In this paper, we present a
set of extensions to MPI that allow communication capabilities to be restored, while maintaining the
extreme level of perfor- mance to which MPI users have become accustomed. The motivation behind the
design choices are weighted against alternatives, a task that requires simultaneously considering
MPI from the viewpoint of both the user and the implemen- tor. The usability of the interfaces for
expressing advanced recovery techniques is then discussed, including the difficult issue of enabling
separate software layers to coordinate their recovery.

---

### document
**Full text:** [document.txt](references/fault-tolerance/document.txt)

. Modern computation is becoming complex in a way that the resource  requirement is gradually
increasing. High Throughput Computing is one  technique to deal with such a complexity. After a
significant amount of time,  computing clusters gets highly overloaded resulting in degradation of
performance. Since there is no central coordinator in Computer Supported  Cooperative Working (CSCW)
load-balancing is more complex.

---

### gong2015
**Full text:** [gong2015.txt](references/fault-tolerance/gong2015.txt)

In this paper, we propose monetary cost optimizations for MPI- based applications with deadline
constraints on Amazon EC2. Par- ticularly, we consider to utilize two kinds of Amazon EC2 instances
(on-demand and spot instances). As a spot instance can fail at any time due to out-of-bid events,
fault tolerant executions are necessary. Through detailed studies, we have found that two common
fault tolerant mechanisms, i.e., checkpoints and replicated executions, are complementary for cost-
effective MPI executions on spot instances. We formulate the optimization problem and propose a
novel cost model to minimize the expected monetary cost.

---

### hpdc14
**Full text:** [hpdc14.txt](references/fault-tolerance/hpdc14.txt)

The use of clouds to execute high-performance computing (HPC) applications has greatly increased
recently. Clouds provide several potential advantages over traditional super- computers and in-house
clusters. The most popular cloud is currently Amazon EC2, which provides a ﬁxed-cost option (called
on-demand) and a variable-cost, auction-based op- tion (called the spot market). The spot market
trades lower cost for potential interruptions that necessitate checkpoint- ing; if the market price
exceeds the bid price, a node is taken away from the user without warning. We explore techniques to
maximize performance per dol- lar given a time constraint within which an application must complete.

---

### moody2010
**Full text:** [moody2010.txt](references/fault-tolerance/moody2010.txt)

High-performance computing (HPC) systems are growing more powerful by utilizing more hardware compo-
nents. As the system mean-time-before-failure correspondingly drops, applications must checkpoint
more frequently to make progress. However, as the system memory sizes grow faster than the bandwidth
to the parallel ﬁle system, the cost of checkpointing begins to dominate application run times.
Multi-level checkpointing potentially solves this problem through multiple types of checkpoints with
different costs and different levels of resiliency in a single run. This solution employs
lightweight checkpoints to handle the most common failure modes and relies on more expensive
checkpoints for less common, but more severe failures.

---

### posner2020
**Full text:** [posner2020.txt](references/fault-tolerance/posner2020.txt)

the probability of permanent hardware failures increases with machine size. A typical resilience
approach to fail/stop failures application-level. Both levels come in many variants, but they
required, full program states are saved, and after a failure the trast, on application-level, only
user-deﬁned data are check- checkpoints parallel programs transparently and restarts them clude task
descriptors and interim results, and are written at regular time intervals and at certain events,
e.g. work stealing. mulas for predicting running times, including failure handling.

---

### s2t4
**Full text:** [s2t4.txt](references/fault-tolerance/s2t4.txt)

W. Bland, A. Bouteiller, T. Herault, J. Hursey, G.

---

### wang2007
**Full text:** [wang2007.txt](references/fault-tolerance/wang2007.txt)

Checkpoint/restart (C/R) has become a requirement for long-running jobs in large-scale clusters due
to a mean- time-to-failure (MTTF) in the order of hours. After a failure, C/R mechanisms generally
require a complete restart of an MPI job from the last checkpoint. A complete restart, how- ever, is
unnecessary since all but one node are typically still alive. Furthermore, a restart may result in
lengthy job re- queuing even though the original job had not exceeded its time quantum. In this
paper, we overcome these shortcomings.

---

## Nas Parallel Benchmarks

### npb
**Full text:** [npb.txt](references/nas-parallel-benchmarks/npb.txt)

A new set of benchmarks has been developed for the performance eval- uation of highly parallel
supercomputers. These benchmarks consist of ﬁve parallel kernels and three simulated application
benchmarks. Together they mimic the computation and data movement characteristics of large scale
computational ﬂuid dynamics (CFD) applications. The principal distinguishing feature of these
benchmarks is their “pencil and paper” speciﬁcation—all details of these benchmarks are speciﬁed
only algorithmically. In this way many of the diﬃculties associated with conven- tional benchmarking
approaches on highly parallel systems are avoided.

---

## Project Documents

### TCC Proposal
**Full text:** [TCC-Proposal.txt](references/project-documents/TCC-Proposal.txt)

Análise de Viabilidade de Instâncias Burstable e Implementação de Tolerância a Falhas em Instâncias
Spot para Computação de Alto Análise de Viabilidade de Instâncias Burstable e Implementação de
Tolerância a Falhas em Instâncias Spot para Computação de Alto Análise de Viabilidade de Instâncias
Burstable e Implementação - Para cada critério avaliado, assinale um X na coluna SIM apenas se
considerado aprovado. Caso contrário, indique as alterações necessárias na coluna Observação.
Problema de Pesquisa . . .

---

## Scientific Initiation

### Avaliacao Preliminar do Desempenho e Custo Financeiro de Aplicacoes de HPC em Clusters de Instancias Burstable da AWS
**Full text:** [Avaliacao_Preliminar_do_Desempenho_e_Custo_Financeiro_de_Aplicacoes_de_HPC_em_Clusters_de_Instancias_Burstable_da_AWS.txt](references/scientific-initiation/Avaliacao_Preliminar_do_Desempenho_e_Custo_Financeiro_de_Aplicacoes_de_HPC_em_Clusters_de_Instancias_Burstable_da_AWS.txt)

Resumo. Instˆancias Burstable da Amazon Web Services (AWS) oferecem de- sempenho vari´avel com base
no consumo de cr´editos de CPU, o que pode re- presentar desafios na avaliac¸˜ao de custo-benef´ıcio
para cargas de trabalho in- tensivas. Este trabalho analisa o desempenho e o custo financeiro de
aplicac¸˜oes HPC em clusters com instˆancias non-Burstable e Burstable. Os resultados mos- tram que
a escolha do tipo de instˆancia ideal depende do perfil da carga de trabalho, com non-Burstable
sendo mais eficiente para cargas intensivas e Burs- A Computac¸˜ao em Nuvem tem desempenhado um
papel essencial na evoluc¸˜ao da Computac¸˜ao de Alto Desempenho (HPC), oferecendo acesso a recursos
escal´aveis e econˆomicos para diversas aplicac¸˜oes. Entre as ofertas dispon´ıveis, as instˆancias
Burs- table da Amazon Web Services (AWS) tˆem ganhado destaque devido `a sua capacidade de fornecer
desempenho vari´avel com base no consumo de cr´editos de CPU.

---

### Performance and Cost Analysis of AWS BurstableInstances for HPC with NAS Parallel Benchmarks
**Full text:** [Performance_and_Cost_Analysis_of_AWS_BurstableInstances_for_HPC_with_NAS_Parallel_Benchmarks.txt](references/scientific-initiation/Performance_and_Cost_Analysis_of_AWS_BurstableInstances_for_HPC_with_NAS_Parallel_Benchmarks.txt)

Traditional High Performance Computing (HPC) typically requires expensive and complex physical
infrastruc- tures. With the advancement of cloud computing, providers such as Amazon Web Services
(AWS) have begun to offer more economically viable alternatives, such as burstable and spot
instances. However, selecting the appropriate resources remains challenging due to performance
variability and the costs associated with different workload profiles. In this context, this work
evaluates the feasibility of using burstable instances for HPC. To this end, an in-depth study was
carried out using AWS on-demand burstable and non-burstable instances, evaluating their performance
and cost under different HPC application scenarios.

---

### projeto pibic
**Full text:** [projeto-pibic.txt](references/scientific-initiation/projeto-pibic.txt)

Projeto submetido ao Programa Institucional de Bolsas de Ini- Pesquisadores das áreas das
Engenharias, Física, Química, Biologia, Geologia, Medicina, entre outras, desenvolvem modelos,
métodos computacionais e simulações numéricas em suas pesquisas científicas. Tendo em vista o
elevado poder de processamento necessário para execução destas aplicações, infraestruturas físicas
de grande porte (clusters ou data- centers) para Computação de Alto Desempenho (High Performance
Computing - HPC) são utilizadas. Estas infraestruturas estão normalmente disponíveis apenas
localmente nos centros de desenvolvimento das pesquisas, com restrições de acesso externo, limitando
o compartilhamento destes recursos computacionais. Além do alto custo de manutenção, os clusters
também exigem dos pesquisados conhecimentos aprofundados sobre aspectos técnicos da infraestrutura
para executarem suas aplicações ou para realizarem atualização neste sistema computacional. Por
outro lado, o paradigma da Computação em Nuvem democratizou o acesso a grandes infraestruturas de
hardware (data-centers) para milhões de organizações e indivíduos com poucos recursos capitais.

---
