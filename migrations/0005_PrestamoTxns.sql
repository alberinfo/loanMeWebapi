CREATE TYPE AcceptedBlockchains AS ENUM('monero')

CREATE TABLE PrestamoTxns(
    "fkPrestamo" BIGINT REFERENCES Prestamo(ID) NOT NULL,
    blockchain AcceptedBlockchains NOT NULL,
    "txnId" TEXT NOT NULL,
    "creationDate" TIMESTAMP WITHOUT TIME ZONE NOT NULL
)