# Adicionar tipos de relações

O catálogo do núcleo em `crates/core/src/catalog.rs` é o ponto de entrada para vínculos paramétricos, vínculos limitantes e harmonias. A interface lê o catálogo; não mantenha listas paralelas de nomes, parâmetros e receitas na interface.

## Contrato comum

Cada tipo tem `TypeInfo`: ID estável, nome, descrição e ícone. Nomes e descrições passam por `i18n::tr`; acrescente as traduções em `crates/desktop/src/i18n.rs`. Os ícones usam nomes de `icons.rs`, nunca caracteres de fonte.

`NumberParameter` define rótulo, mínimo, máximo, valor inicial, unidade e sugestões. Sugestões não substituem a validação. A matemática e as restrições devem permanecer no núcleo, disponíveis mesmo sem a interface.

O catálogo é estático e compilado com o programa. Ele não carrega código externo ou plugins. Todos os construtores retornam `Result<Command>`; a interface só aplica o comando na confirmação. Conversões e criação continuam dentro de uma única `Command::Batch`, com desfazer/refazer e reversão em caso de erro.

## Vínculo paramétrico

Registre um `ParametricType` em `PARAMETRIC_TYPES`, com seus canais e construtor. `CHANNELS` fornece os índices estáveis R/G/B/H/S/V, nome completo, letra e modelo. O editor gera os botões em duas linhas a partir desses dados. O seletor de tipo aparece automaticamente quando há mais de um tipo registrado.

Para uma nova receita que use as restrições existentes, seu construtor pode retornar `Command::LinkColors` ou um `Command::Batch`. O núcleo valida cores distintas, canais, perfis e compatibilidade. Por exemplo, um preset de matiz pode retornar:

```rust
build: |ids, _| Ok(Command::LinkColors { ids, channels: vec![3] }),
```

Para novas equações ou canais fora dos seis existentes, implemente primeiro o modelo e a operação correspondentes no núcleo e no solver. Não simule restrições novas apenas com controles visuais. O editor atual de canais representa RGB/HSV.

## Vínculo limitante

Registre um `LimitType` em `LIMIT_TYPES`. A coluna de tipos é gerada automaticamente. Defina o parâmetro numérico, a função de medição, seu rótulo, a explicação e o construtor do comando. O editor gera o slider, entrada numérica, sugestões e valor medido. O tipo atual `contrast` referencia `color::contrast` e `Command::AddContrast`.

Uma restrição realmente nova precisa também de representação persistida, validação e verificação em `solver::problems` e na solução das edições. Acrescente uma migração somente se o formato exigir. Um simples registro no catálogo não implementa a imposição matemática de uma restrição inédita. Nunca reutilize o ID `contrast` para uma regra diferente.

## Harmonia

Adicione um `HarmonyType` em `HARMONY_TYPES`. O gerador recebe a cor base, abertura e quantidade e retorna as cores geradas. Defina `spread` ou `count` apenas quando o parâmetro for aplicável. A lista, nomes, descrições, prévia, controles e edição da abertura no painel Vínculos usam esse mesmo registro.

Exemplo de uma receita fixa de três matizes, usando o helper interno `angles`:

```rust
harmony!("triad.custom", "Tríade personalizada", "Três matizes com intervalos próprios.",
    None, None, |base, _, _| angles(base, &[0.0, 100.0, 230.0])),
```

As harmonias existentes congelam relações afins HSV entre seus participantes. Geradores que precisem de comportamento diferente depois da criação exigem uma extensão explícita desse contrato, incluindo solver e persistência. O gradiente continua sendo uma composição distinta com suas próprias regras de interpolação.

## Validação de uma extensão

- ID único e estável; metadados traduzidos; parâmetros e sugestões dentro dos limites.
- Prévia somente de leitura, originais preservadas e diagnóstico de conflitos com IDs de relações.
- Aplicação, erro, cancelamento e desfazer/refazer atômicos.
- Propagação a partir de participantes diferentes, bloqueios, ciclos e fontes vinculadas.
- Gravação/reabertura e migração, se o modelo persistido mudar.
- Seletor e controles em português/inglês, temas claro/escuro e janela reduzida.

O catálogo atual mantém os IDs, fórmulas e formatos persistidos existentes. Novas restrições matemáticas devem acrescentar testes de comportamento do núcleo; alterações de interface não podem enfraquecer a validação.

## Diagnósticos para o painel Conflitos

A versão 0.1.13 acrescenta `colorcraft_core::diagnostics`. `Diagnostic` contém operação, origens, canal/direção da edição, valores solicitados, cadeias de `Hop` e motivos `Cause`. Os snapshots de cores e `RelationEvidence` preservam a identificação de restrições presentes apenas em candidatos que foram rejeitados. Esses tipos não fazem parte do schema do projeto.

`solver::Blocked` e `composition::CompositionIssue` conservam suas mensagens e IDs de relações e agora incluem `diagnostic: Option<Box<Diagnostic>>`. As assinaturas de `solve` e `edit` permanecem disponíveis. `edit_diagnosed` preserva também o diagnóstico quando a edição contínua termina num limite, mesmo retornando sucesso com o valor limitado. `Engine::last_diagnostic` recebe o diagnóstico da operação mais recente; o desktop consome-o após a ação, sem interpretar mensagens.

Ao implementar uma nova restrição:

- Emita um motivo tipado com IDs dos participantes e da relação, e os parâmetros que explicam o bloqueio. Não use texto traduzido como código de erro.
- Conserve a procedência: cada salto precisa corresponder à restrição que transmite a alteração. Para canais afins, o solver registra as cores que impõem os extremos e o canal envolvido; gradientes têm saltos direcionados da origem à intermediária. Ciclos são explicados coletivamente com suas relações responsáveis.
- Acrescente classificação e metadados em `Diagnostic::annotate` para que o item continue legível quando a criação falhar e o candidato não existir no projeto. `Cause` precisa fornecer suas referências por `colors()`/`relations()` e uma identidade estável por `key()`.
- Não emita eventos para cada sondagem numérica ou chamada de renderização. O solver devolve evidências; o desktop registra somente ações explícitas do usuário. Novos tipos devem cobrir bloqueio, limites, erro sem mutação, prévia e desfazer/refazer nos testes.

A identidade do histórico não usa valores numéricos do arraste. Relações já existentes conservam o ID; criações recusadas usam uma identidade transitória baseada no tipo e participantes, pois seus IDs candidatos mudam entre tentativas. A apresentação, textos traduzidos e menu de remoção ficam no desktop; o núcleo não altera cores para produzir explicações.
