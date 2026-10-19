# Preparação da proposta inicial de merge — ColorCraft

Decisão de escopo: 19/10/2026. A primeira proposta de merge terá **somente o aplicativo ColorCraft**. O fork [EduMzz/vectorcraft-colorcraft](https://github.com/EduMzz/vectorcraft-colorcraft) fica como prova de conceito (PoC) para demonstrar como programas da suíte podem consumir e enviar cores. Suas mudanças no VectorCraft não fazem parte dessa proposta.

Esta documentação prepara o recorte de revisão; não significa que um PR já foi aberto, que o projeto foi aceito na suíte ou que o protocolo foi adotado como padrão pelos demais aplicativos.

## Conteúdo da proposta

- Aplicativo independente, edição de paletas e relações, fontes/capturas, metadados, histórico, interface e preferências.
- Modelo do projeto, migrações, persistência, recuperação, publicação local e contratos de exportação.
- Receptor local de integração, abertura por URI, registro por usuário e testes do contrato. Esses recursos não exigem instalar ou executar o fork do VectorCraft.
- Fontes, manifests, lockfile, documentação, testes, scripts de build/pacote e avisos de licença necessários para reproduzir o ColorCraft.

O protocolo e a biblioteca publicada devem ser revisados como interfaces do ColorCraft. A existência do cliente experimental não torna seu modelo de documento ou sua implementação interna requisitos para outros programas.

## Conteúdo que fica no PoC

O painel Amostras do ColorCraft no VectorCraft, a extensão `colorcraft` de seus documentos, seus comandos, o mapeamento para amostras globais e seus pacotes de avaliação permanecem no fork. Uma futura integração em cada aplicativo terá proposta própria, após a revisão do ColorCraft, com recorte e validação específicos.

## Preparação antes de submeter

1. Definir com os mantenedores o repositório, a branch de destino e as convenções de contribuição. O destino não está definido neste documento; não presumir um merge no repositório do VectorCraft.
2. Separar o diretório `colorcraft/` do Colorator web e de artefatos locais. Não incluir `target/`, `dist/`, caches de licenças, logs, dados de recuperação, bibliotecas pessoais ou projetos usados pelo usuário.
3. Revisar compatibilidade, esquema dos projetos, contratos publicados, limites e isolamento do receptor. Usar [formatos](FORMATOS.md), [contrato local](BRIDGE-v1.md) e os testes como evidência.
4. Reexecutar formatação, Clippy, testes e builds pertinentes no commit candidato; distinguir resultados do ColorCraft dos resultados do cliente PoC. As evidências atuais estão em [validação](VALIDACAO.md) e [integração](BRIDGE-VALIDATION.md).
5. Conferir dependências e proveniência. O código tem licenciamento MIT OU Apache-2.0; as ilustrações fornecidas pelo autor têm termos separados em [ASSETS.md](../assets/ASSETS.md). O uso no aplicativo foi autorizado, mas uma licença geral para reutilização dessas imagens ainda não foi declarada. Registrar os termos necessários à distribuição pretendida antes da submissão; não presumir que a licença do código as abrange.
6. Preparar uma descrição centrada no ColorCraft, incluindo escopo, compatibilidade, validação e limitações. Referenciar o fork apenas como demonstração opcional de interoperabilidade.

Os pacotes Windows existentes continuam sendo entregas de avaliação. Não reorganizar instalações, preferências ou instâncias do usuário para preparar a proposta.
