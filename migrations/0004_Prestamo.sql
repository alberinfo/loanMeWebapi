CREATE TYPE AcceptedBlockchains AS ENUM('Monero', 'Bitcoin', 'BitcoinTestnet', 'Ethereum', 'EthereumTestnet');

CREATE TABLE Prestamo(
    ID BIGSERIAL PRIMARY KEY NOT NULL,
    monto DECIMAL NOT NULL CHECK (monto > 0),
    "fechaCreacion" TIMESTAMP WITHOUT TIME ZONE NOT NULL,
    interes FLOAT NOT NULL,
    "plazoPago" TIMESTAMP WITHOUT TIME ZONE NOT NULL, 
    "intervaloPago" TEXT NOT NULL,
    riesgo INT NOT NULL CHECK (riesgo > 0),
    "walletId" TEXT NOT NULL,
    "walletChain" AcceptedBlockchains NOT NULL,
    "fkPrestatario" BIGINT REFERENCES Usuario(ID),
    "fkPrestamista" BIGINT REFERENCES Usuario(ID),
    CHECK ("fkPrestatario" IS NOT NULL OR "fkPrestamista" IS NOT NULL)
)