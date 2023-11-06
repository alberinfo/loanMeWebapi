CREATE TABLE PrestamoTxns(
    "fkPrestamo" BIGINT REFERENCES Prestamo(ID) NOT NULL,
    "txnId" TEXT NOT NULL
);